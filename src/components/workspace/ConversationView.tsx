import { useState } from "react";

import type { TypedCommand } from "../../generated/session-types";
import { useSessionStore } from "../../store/sessionStore";
import { PendingActionCard } from "./PendingActionCard";

function describeCommand(cmd: TypedCommand): string {
  if (cmd.type === "graphEdit") {
    return cmd.payload.type;
  }
  return cmd.payload.type;
}

type AgentRuntimeDiagnosticsProps = {
  providerStatus: string;
  runtimeConnectionStatus: string;
  pendingCount: number;
  blockedReason: string | null;
};

export function AgentRuntimeDiagnostics({
  providerStatus,
  runtimeConnectionStatus,
  pendingCount,
  blockedReason,
}: AgentRuntimeDiagnosticsProps) {
  return (
    <div className="sidecar-section agent-runtime-diagnostics">
      <div className="diagnostic-row">
        <span>Provider</span>
        <strong>{providerStatus}</strong>
      </div>
      <div className="diagnostic-row">
        <span>Runtime</span>
        <strong>{runtimeConnectionStatus}</strong>
      </div>
      <div className="diagnostic-row">
        <span>Approvals</span>
        <strong>{pendingCount} waiting</strong>
      </div>
      {blockedReason ? (
        <p className="diagnostic-detail">{blockedReason}</p>
      ) : null}
    </div>
  );
}

export function ConversationView() {
  const [inputValue, setInputValue] = useState("");

  const {
    session,
    conversationMessages,
    pendingActions,
    agentFrozen,
    agentRuntime,
    isLoading,
    sendAgentMessage,
    toggleFreezeAgent,
    reclaimOwnership,
    approvePendingAction,
    rejectPendingAction,
  } = useSessionStore();

  const handleSend = () => {
    const trimmed = inputValue.trim();
    if (!trimmed || isLoading) return;
    setInputValue("");
    void sendAgentMessage(trimmed);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const pendingQueue = pendingActions.filter((a) => a.status === "pending");
  const reviewedQueue = pendingActions.filter((a) => a.status !== "pending").slice(-3).reverse();
  const pendingCountLabel = `${pendingQueue.length} pending`;
  const agentRuntimeStatus = agentRuntime?.status ?? (agentFrozen ? "Frozen" : "Available");
  const agentRuntimeRef = session?.runtimeStatus.find((runtime) => runtime.runtime === "agent") ?? null;
  const providerStatus = agentRuntime?.isAvailable === false ? "Provider unavailable" : "Provider available";
  const runtimeConnectionStatus = agentRuntimeRef
    ? `${agentRuntimeRef.status}${agentRuntimeRef.targetId ? ` / ${agentRuntimeRef.targetId}` : ""}`
    : "local planner";
  const blockedReason = agentRuntime?.isAvailable === false
    ? agentRuntimeRef?.lastError ?? "Agent provider is unavailable; proposals cannot be generated."
    : agentFrozen
      ? "Agent is frozen by performer override; new commands stay parked until unfrozen."
      : agentRuntimeRef?.lastError ?? null;

  return (
    <section className="conversation-view" aria-label="Agent collaborator sidecar">
      <div className="conversation-main">
        <div className="panel-heading conversation-heading">
          <div>
            <p className="eyebrow">Collaborator</p>
            <h2>Agent sidecar</h2>
          </div>
          <span className="queue-count-pill">{pendingCountLabel}</span>
        </div>

        {blockedReason ? (
          <div className="frozen-banner">{blockedReason}</div>
        ) : null}

        <div className="conversation-messages">
          {conversationMessages.length === 0 ? (
            <p className="empty-state">
              Ask for a graph, routing, or performance change.
            </p>
          ) : (
            conversationMessages.map((msg) => (
              <div key={msg.id} className={`message-bubble message-${msg.role}`}>
                <span className="message-role">{msg.role === "user" ? "You" : "Agent"}</span>
                <p className="message-content">{msg.content}</p>
                {msg.intent ? (
                  <div className="intent-preview">
                    <span className="intent-confidence">
                      Confidence: {(msg.intent.confidence * 100).toFixed(0)}%
                    </span>
                    {msg.intent.parsedCommands.map((cmd, i) => (
                      <div key={i} className="command-preview">
                        <code>{cmd.type}: {describeCommand(cmd)}</code>
                      </div>
                    ))}
                  </div>
                ) : null}
              </div>
            ))
          )}
        </div>

        <div className="conversation-input-row">
          <input
            type="text"
            className="conversation-input"
            placeholder="Type a command..."
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            onKeyDown={handleKeyDown}
            disabled={isLoading}
          />
          <button
            type="button"
            onClick={handleSend}
            disabled={isLoading || !inputValue.trim()}
          >
            Send
          </button>
        </div>
      </div>

      <aside className="conversation-sidecar" aria-label="Pending agent actions">
        <div className="sidecar-section conversation-safety-strip">
          <div>
            <p className="eyebrow">Safety</p>
            <strong>{agentRuntimeStatus}</strong>
          </div>
          <button
            type="button"
            className={agentFrozen ? "freeze-button-active" : ""}
            onClick={() => void toggleFreezeAgent()}
            disabled={isLoading}
          >
            {agentFrozen ? "Unfreeze Agent" : "Freeze Agent"}
          </button>
          <button
            type="button"
            onClick={() => void reclaimOwnership()}
            disabled={isLoading}
          >
            Reclaim All
          </button>
        </div>

        <AgentRuntimeDiagnostics
          providerStatus={providerStatus}
          runtimeConnectionStatus={runtimeConnectionStatus}
          pendingCount={pendingQueue.length}
          blockedReason={blockedReason}
        />

        <div className="sidecar-section pending-actions">
          <div className="pending-actions-heading">
            <p className="eyebrow">Plan Review</p>
            <strong>{pendingQueue.length}</strong>
          </div>
          {pendingQueue.length === 0 ? (
            <p className="empty-state compact-empty-state">No approvals waiting.</p>
          ) : (
            <div className="pending-action-list">
              {pendingQueue.map((action) => (
                <PendingActionCard
                  key={action.id}
                  action={action}
                  session={session}
                  isLoading={isLoading}
                  onApprove={() => void approvePendingAction(action.id)}
                  onReject={() => void rejectPendingAction(action.id)}
                />
              ))}
            </div>
          )}
          {reviewedQueue.length > 0 ? (
            <div className="reviewed-actions">
              <p className="eyebrow">Recent Decisions</p>
              {reviewedQueue.map((action) => (
                <PendingActionCard
                  key={action.id}
                  action={action}
                  session={session}
                  isLoading={isLoading}
                  onApprove={() => void approvePendingAction(action.id)}
                  onReject={() => void rejectPendingAction(action.id)}
                />
              ))}
            </div>
          ) : null}
        </div>
      </aside>
    </section>
  );
}
