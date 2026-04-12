import { useState } from "react";

import type { ActionHistoryEntry, TypedCommand } from "../../generated/session-types";

function describeCommand(cmd: TypedCommand): string {
  if (cmd.type === "graphEdit") {
    return cmd.payload.type;
  }
  return cmd.payload.type;
}

type ActivityFilter = "all" | "user" | "agent";

type ActivityPanelProps = {
  actionHistory: ActionHistoryEntry[];
};

export function ActivityPanel({ actionHistory }: ActivityPanelProps) {
  const [filter, setFilter] = useState<ActivityFilter>("all");

  const filtered = filter === "all"
    ? actionHistory
    : actionHistory.filter((entry) => entry.actor.actorId === filter);

  return (
    <section className="activity-panel">
      <div className="panel-heading">
        <p className="eyebrow">Activity</p>
        <div className="activity-filters">
          {(["all", "user", "agent"] as const).map((value) => (
            <button
              key={value}
              type="button"
              className={`view-tab ${filter === value ? "view-tab-active" : ""}`}
              onClick={() => setFilter(value)}
            >
              {value === "all" ? "All" : value === "user" ? "User" : "Agent"}
            </button>
          ))}
        </div>
      </div>
      <div className="activity-list">
        {filtered.length === 0 ? (
          <p className="empty-state">No actions recorded yet.</p>
        ) : (
          filtered.map((entry) => (
            <div key={entry.id} className="activity-entry">
              <div className="activity-entry-header">
                <span className={`actor-badge actor-${entry.actor.actorId}`}>
                  {entry.actor.actorId}
                </span>
                <span className="activity-timestamp">
                  {new Date(entry.timestamp).toLocaleTimeString()}
                </span>
              </div>
              <p className="activity-description">{entry.diff.description}</p>
              <div className="activity-detail">
                <span className="activity-command">
                  {entry.command.type}: {describeCommand(entry.command)}
                </span>
                {entry.diff.affectedNodeIds.length > 0 ? (
                  <span className="affected-nodes">
                    Nodes: {entry.diff.affectedNodeIds.join(", ")}
                  </span>
                ) : null}
              </div>
            </div>
          ))
        )}
      </div>
    </section>
  );
}
