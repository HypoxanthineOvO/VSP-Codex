use std::any::Any;
use std::sync::Arc;
use std::time::Duration;

use codex_protocol::ThreadId;
use codex_protocol::models::BaseInstructions;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::RolloutItem;
use codex_protocol::protocol::SessionSource;
use codex_protocol::protocol::ThreadHistoryMode;
use codex_protocol::protocol::ThreadMemoryMode;
use codex_protocol::protocol::UserMessageEvent;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::time::timeout;
use uuid::Uuid;

use super::LiveThread;
use crate::AppendThreadItemsParams;
use crate::ArchiveThreadParams;
use crate::CreateThreadParams;
use crate::DeleteThreadParams;
use crate::InMemoryThreadStore;
use crate::ItemPage;
use crate::ListItemsParams;
use crate::ListThreadsParams;
use crate::ListTurnsParams;
use crate::LoadThreadHistoryParams;
use crate::ReadThreadByRolloutPathParams;
use crate::ReadThreadParams;
use crate::ResumeThreadParams;
use crate::SearchThreadsParams;
use crate::StoredThread;
use crate::StoredThreadHistory;
use crate::ThreadPage;
use crate::ThreadPersistenceMetadata;
use crate::ThreadSearchPage;
use crate::ThreadStore;
use crate::ThreadStoreError;
use crate::ThreadStoreFuture;
use crate::ThreadStoreResult;
use crate::TurnPage;
use crate::UpdateThreadMetadataParams;

const TEST_TIMEOUT: Duration = Duration::from_secs(1);

struct MetadataUpdateAttempt {
    params: UpdateThreadMetadataParams,
    completion: oneshot::Sender<ThreadStoreResult<()>>,
}

struct ControlledThreadStore {
    inner: InMemoryThreadStore,
    update_sender: mpsc::UnboundedSender<MetadataUpdateAttempt>,
}

impl ControlledThreadStore {
    fn new() -> (Arc<Self>, mpsc::UnboundedReceiver<MetadataUpdateAttempt>) {
        let (update_sender, update_receiver) = mpsc::unbounded_channel();
        (
            Arc::new(Self {
                inner: InMemoryThreadStore::default(),
                update_sender,
            }),
            update_receiver,
        )
    }
}

impl ThreadStore for ControlledThreadStore {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn create_thread(&self, params: CreateThreadParams) -> ThreadStoreFuture<'_, ()> {
        ThreadStore::create_thread(&self.inner, params)
    }

    fn resume_thread(&self, params: ResumeThreadParams) -> ThreadStoreFuture<'_, ()> {
        ThreadStore::resume_thread(&self.inner, params)
    }

    fn append_items(&self, params: AppendThreadItemsParams) -> ThreadStoreFuture<'_, ()> {
        ThreadStore::append_items(&self.inner, params)
    }

    fn persist_thread(&self, thread_id: ThreadId) -> ThreadStoreFuture<'_, ()> {
        ThreadStore::persist_thread(&self.inner, thread_id)
    }

    fn flush_thread(&self, thread_id: ThreadId) -> ThreadStoreFuture<'_, ()> {
        ThreadStore::flush_thread(&self.inner, thread_id)
    }

    fn shutdown_thread(&self, thread_id: ThreadId) -> ThreadStoreFuture<'_, ()> {
        ThreadStore::shutdown_thread(&self.inner, thread_id)
    }

    fn discard_thread(&self, thread_id: ThreadId) -> ThreadStoreFuture<'_, ()> {
        ThreadStore::discard_thread(&self.inner, thread_id)
    }

    fn load_history(
        &self,
        params: LoadThreadHistoryParams,
    ) -> ThreadStoreFuture<'_, StoredThreadHistory> {
        ThreadStore::load_history(&self.inner, params)
    }

    fn read_thread(&self, params: ReadThreadParams) -> ThreadStoreFuture<'_, StoredThread> {
        ThreadStore::read_thread(&self.inner, params)
    }

    fn read_thread_by_rollout_path(
        &self,
        params: ReadThreadByRolloutPathParams,
    ) -> ThreadStoreFuture<'_, StoredThread> {
        ThreadStore::read_thread_by_rollout_path(&self.inner, params)
    }

    fn list_threads(&self, params: ListThreadsParams) -> ThreadStoreFuture<'_, ThreadPage> {
        ThreadStore::list_threads(&self.inner, params)
    }

    fn search_threads(
        &self,
        params: SearchThreadsParams,
    ) -> ThreadStoreFuture<'_, ThreadSearchPage> {
        ThreadStore::search_threads(&self.inner, params)
    }

    fn list_turns(&self, params: ListTurnsParams) -> ThreadStoreFuture<'_, TurnPage> {
        ThreadStore::list_turns(&self.inner, params)
    }

    fn list_items(&self, params: ListItemsParams) -> ThreadStoreFuture<'_, ItemPage> {
        ThreadStore::list_items(&self.inner, params)
    }

    fn update_thread_metadata(
        &self,
        params: UpdateThreadMetadataParams,
    ) -> ThreadStoreFuture<'_, StoredThread> {
        Box::pin(async move {
            let (completion, result) = oneshot::channel();
            self.update_sender
                .send(MetadataUpdateAttempt {
                    params: params.clone(),
                    completion,
                })
                .map_err(|_| ThreadStoreError::Internal {
                    message: "metadata update test receiver dropped".to_string(),
                })?;
            result.await.map_err(|err| ThreadStoreError::Internal {
                message: format!("metadata update test completion dropped: {err}"),
            })??;
            ThreadStore::update_thread_metadata(&self.inner, params).await
        })
    }

    fn archive_thread(&self, params: ArchiveThreadParams) -> ThreadStoreFuture<'_, ()> {
        ThreadStore::archive_thread(&self.inner, params)
    }

    fn unarchive_thread(&self, params: ArchiveThreadParams) -> ThreadStoreFuture<'_, StoredThread> {
        ThreadStore::unarchive_thread(&self.inner, params)
    }

    fn delete_thread(&self, params: DeleteThreadParams) -> ThreadStoreFuture<'_, ()> {
        ThreadStore::delete_thread(&self.inner, params)
    }
}

#[tokio::test]
async fn append_does_not_wait_for_blocked_metadata_persistence() {
    let (store, mut updates) = ControlledThreadStore::new();
    let live_thread = create_live_thread(Arc::clone(&store)).await;

    timeout(
        TEST_TIMEOUT,
        live_thread.append_items(&[user_message("first message")]),
    )
    .await
    .expect("append should not wait for state metadata persistence")
    .expect("append should succeed");
    let attempt = next_update(&mut updates).await;

    assert_eq!(store.inner.calls().await.append_items, 1);
    assert_eq!(store.inner.calls().await.update_thread_metadata, 0);
    assert_eq!(
        attempt.params.patch.preview.as_deref(),
        Some("first message")
    );

    attempt
        .completion
        .send(Ok(()))
        .expect("metadata worker should still be waiting");
    live_thread
        .flush()
        .await
        .expect("flush should drain metadata");
    assert_eq!(store.inner.calls().await.update_thread_metadata, 1);
}

#[tokio::test]
async fn metadata_writes_coalesce_into_one_bounded_queued_generation() {
    let (store, mut updates) = ControlledThreadStore::new();
    let live_thread = create_live_thread(Arc::clone(&store)).await;

    live_thread
        .append_items(&[user_message("first message")])
        .await
        .expect("first append should succeed");
    let first_attempt = next_update(&mut updates).await;

    for index in 0..100 {
        live_thread
            .append_items(&[user_message(&format!("queued message {index}"))])
            .await
            .expect("queued append should succeed");
    }
    {
        let state = live_thread.metadata_worker.state.lock().await;
        assert!(state.queued_update.is_some());
        assert!(state.running);
    }

    first_attempt
        .completion
        .send(Ok(()))
        .expect("first metadata update should still be waiting");
    let coalesced_attempt = next_update(&mut updates).await;
    coalesced_attempt
        .completion
        .send(Ok(()))
        .expect("coalesced metadata update should still be waiting");
    live_thread
        .flush()
        .await
        .expect("flush should drain coalesced metadata");

    let calls = store.inner.calls().await;
    assert_eq!(calls.append_items, 101);
    assert_eq!(calls.update_thread_metadata, 2);
}

async fn create_live_thread(store: Arc<ControlledThreadStore>) -> LiveThread {
    let thread_id = ThreadId::new();
    let thread_store: Arc<dyn ThreadStore> = store;
    LiveThread::create(thread_store, create_params(thread_id))
        .await
        .expect("live thread should be created")
}

fn create_params(thread_id: ThreadId) -> CreateThreadParams {
    CreateThreadParams {
        session_id: thread_id.into(),
        thread_id,
        extra_config: None,
        forked_from_id: None,
        parent_thread_id: None,
        source: SessionSource::Exec,
        thread_source: None,
        originator: "test_originator".to_string(),
        base_instructions: BaseInstructions::default(),
        dynamic_tools: Vec::new(),
        selected_capability_roots: Vec::new(),
        multi_agent_version: None,
        history_mode: ThreadHistoryMode::Legacy,
        initial_window_id: Uuid::now_v7().to_string(),
        metadata: ThreadPersistenceMetadata {
            cwd: None,
            model_provider: "test-provider".to_string(),
            memory_mode: ThreadMemoryMode::Enabled,
        },
    }
}

fn user_message(message: &str) -> RolloutItem {
    RolloutItem::EventMsg(EventMsg::UserMessage(UserMessageEvent {
        message: message.to_string(),
        ..Default::default()
    }))
}

async fn next_update(
    updates: &mut mpsc::UnboundedReceiver<MetadataUpdateAttempt>,
) -> MetadataUpdateAttempt {
    timeout(TEST_TIMEOUT, updates.recv())
        .await
        .expect("metadata update should start")
        .expect("metadata update channel should remain open")
}
