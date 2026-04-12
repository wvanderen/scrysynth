type ConversationViewProps = {
  sessionTitle: string;
};

export function ConversationView({ sessionTitle }: ConversationViewProps) {
  return (
    <section className="conversation-view">
      <div className="panel-heading">
        <p className="eyebrow">Conversation</p>
        <h2>Talk to your instrument</h2>
      </div>
      <div className="conversation-placeholder">
        <p className="empty-state">
          Direct the session in natural language. Agent collaboration arrives in a future phase.
        </p>
        <p className="conversation-session-ref">
          Currently loaded: <strong>{sessionTitle}</strong>
        </p>
      </div>
    </section>
  );
}
