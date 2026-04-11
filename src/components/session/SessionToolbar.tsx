type SessionToolbarProps = {
  title: string;
  isLoading: boolean;
  onNewSession: () => void;
  onSaveSession: () => void;
  onOpenSession: () => void;
};

export function SessionToolbar({
  title,
  isLoading,
  onNewSession,
  onSaveSession,
  onOpenSession,
}: SessionToolbarProps) {
  return (
    <header className="session-toolbar">
      <div>
        <p className="eyebrow">Scrysynth Session Core</p>
        <h1>{title}</h1>
      </div>
      <div className="toolbar-actions">
        <button type="button" onClick={onNewSession} disabled={isLoading}>
          New Session
        </button>
        <button type="button" onClick={onSaveSession} disabled={isLoading}>
          Save Session
        </button>
        <button type="button" onClick={onOpenSession} disabled={isLoading}>
          Open Session
        </button>
      </div>
    </header>
  );
}
