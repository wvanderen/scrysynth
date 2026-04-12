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

  return (
    <section className="conversation-view">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">Conversation</p>
          <h2>Talk to your instrument</h2>
        </div>
        <div className="conversation-controls">
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
      </div>

      {agentFrozen ? (
        <div className="frozen-banner">Agent is frozen — commands will not execute.</div>
      ) : null}

      <div className="conversation-messages">
        {conversationMessages.length === 0 ? (
          <p className="empty-state">
            Direct the session in natural language. Try "add an oscillator" or "fade in the delay".
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

      {pendingActions.length > 0 ? (
        <div className="pending-actions">
          <p className="eyebrow">Pending Actions</p>
          {pendingActions
            .filter((a) => a.status === "pending")
            .map((action) => (
              <PendingActionCard
                key={action.id}
                action={action}
                isLoading={isLoading}
                onApprove={() => void approvePendingAction(action.id)}
                onReject={() => void rejectPendingAction(action.id)}
              />
            ))}
        </div>
      ) : null}

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
    </section>
  );
}
