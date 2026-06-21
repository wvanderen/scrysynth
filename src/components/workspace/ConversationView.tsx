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

export function ConversationView() {
  const [inputValue, setInputValue] = useState("");

  const {
    conversationMessages,
    pendingActions,
    agentFrozen,
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
  const pendingCountLabel = `${pendingQueue.length} pending`;

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

        {agentFrozen ? (
          <div className="frozen-banner">Agent is frozen. Commands stay parked.</div>
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
            <strong>{agentFrozen ? "Frozen" : "Agent live"}</strong>
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

        <div className="sidecar-section pending-actions">
          <div className="pending-actions-heading">
            <p className="eyebrow">Pending Actions</p>
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
                  isLoading={isLoading}
                  onApprove={() => void approvePendingAction(action.id)}
                  onReject={() => void rejectPendingAction(action.id)}
                />
              ))}
            </div>
          )}
        </div>
      </aside>
    </section>
  );
}
