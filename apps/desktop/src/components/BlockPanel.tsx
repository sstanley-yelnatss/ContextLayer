import { useEffect, useState } from "react";
import { listBlocksForPicker, saveBlock, softDeleteBlock } from "../api";
import {
  BELIEF_STATES,
  CONFIDENCE_LEVELS,
  placeholdersForTemplate,
  SYSTEM_TAGS,
  hypothesisFieldLabel,
  type BeliefState,
  type BlockEntry,
  type BlockSystemTag,
  type ConfidenceLevel,
  type Workspace,
} from "../types";

function blockRevision(block: BlockEntry): string {
  return JSON.stringify({
    updated_at: block.updated_at,
    title: block.title,
    hypothesis: block.hypothesis?.text ?? "",
    action: block.action?.text ?? "",
    evidence: block.evidence?.text ?? "",
    evidence_source: block.evidence?.source ?? "",
    conclusion: block.conclusion?.text ?? "",
    belief_state: block.belief_state,
    system_tag: block.system_tag,
    user_tag: block.user_tag ?? "",
    linked_block_ids: block.linked_block_ids ?? [],
  });
}

function applyBlockToForm(block: BlockEntry | null, setters: {
  setTitle: (v: string) => void;
  setHypothesisText: (v: string) => void;
  setActionText: (v: string) => void;
  setEvidenceText: (v: string) => void;
  setEvidenceSource: (v: string) => void;
  setConclusionText: (v: string) => void;
  setConclusionOutcome: (v: string) => void;
  setConclusionTag: (v: string) => void;
  setConfidenceLevel: (v: ConfidenceLevel | "") => void;
  setBeliefState: (v: BeliefState) => void;
  setSystemTag: (v: BlockSystemTag) => void;
  setUserTag: (v: string) => void;
  setLinkToBlockIds: (v: string[]) => void;
}) {
  setters.setTitle(block?.title ?? "");
  setters.setHypothesisText(block?.hypothesis?.text ?? "");
  setters.setActionText(block?.action?.text ?? "");
  setters.setEvidenceText(block?.evidence?.text ?? "");
  setters.setEvidenceSource(block?.evidence?.source ?? "");
  setters.setConclusionText(block?.conclusion?.text ?? "");
  setters.setConclusionOutcome(block?.conclusion?.outcome ?? "uncertain");
  setters.setConclusionTag(block?.conclusion?.tag ?? "none");
  setters.setConfidenceLevel((block?.conclusion?.confidence_level as ConfidenceLevel) ?? "");
  setters.setBeliefState(block?.belief_state ?? "open");
  setters.setSystemTag(block?.system_tag ?? "none");
  setters.setUserTag(block?.user_tag ?? "");
  setters.setLinkToBlockIds(block?.linked_block_ids ?? []);
}

interface Props {
  workspace: Workspace;
  block: BlockEntry | null;
  onClose: () => void;
  onSaved: () => void;
}

export default function BlockPanel({ workspace, block, onClose, onSaved }: Props) {
  const isEdit = !!block;
  const ph = placeholdersForTemplate(workspace.template);
  const hypothesisLabel = hypothesisFieldLabel(workspace.template);

  const [title, setTitle] = useState(block?.title ?? "");
  const [hypothesisText, setHypothesisText] = useState(block?.hypothesis?.text ?? "");
  const [actionText, setActionText] = useState(block?.action?.text ?? "");
  const [evidenceText, setEvidenceText] = useState(block?.evidence?.text ?? "");
  const [evidenceSource, setEvidenceSource] = useState(block?.evidence?.source ?? "");
  const [conclusionText, setConclusionText] = useState(block?.conclusion?.text ?? "");
  const [conclusionOutcome, setConclusionOutcome] = useState(
    block?.conclusion?.outcome ?? "uncertain",
  );
  const [conclusionTag, setConclusionTag] = useState(block?.conclusion?.tag ?? "none");
  const [confidenceLevel, setConfidenceLevel] = useState<ConfidenceLevel | "">(
    (block?.conclusion?.confidence_level as ConfidenceLevel) ?? "",
  );
  const [beliefState, setBeliefState] = useState<BeliefState>(
    block?.belief_state ?? "open",
  );
  const [systemTag, setSystemTag] = useState<BlockSystemTag>(
    block?.system_tag ?? "none",
  );
  const [userTag, setUserTag] = useState(block?.user_tag ?? "");
  const [linkToBlockIds, setLinkToBlockIds] = useState<string[]>(
    block?.linked_block_ids ?? [],
  );
  const [pickerBlocks, setPickerBlocks] = useState<[string, string][]>([]);
  const [error, setError] = useState("");

  useEffect(() => {
    listBlocksForPicker(workspace.id).then(setPickerBlocks);
  }, [workspace.id]);

  // Keep form in sync when parent refreshes the same block (e.g. MCP save_block).
  useEffect(() => {
    if (!block) return;
    applyBlockToForm(block, {
      setTitle,
      setHypothesisText,
      setActionText,
      setEvidenceText,
      setEvidenceSource,
      setConclusionText,
      setConclusionOutcome,
      setConclusionTag,
      setConfidenceLevel,
      setBeliefState,
      setSystemTag,
      setUserTag,
      setLinkToBlockIds,
    });
  }, [block?.id, block ? blockRevision(block) : null]);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError("");
    try {
      await saveBlock({
        workspaceId: workspace.id,
        blockId: block?.id,
        title: title.trim() || undefined,
        hypothesisText,
        actionText,
        evidenceText,
        evidenceSource,
        conclusionText,
        conclusionOutcome: conclusionText ? conclusionOutcome : undefined,
        conclusionTag: conclusionText ? conclusionTag : undefined,
        confidenceLevel: confidenceLevel || undefined,
        beliefState,
        systemTag,
        userTag: userTag || undefined,
        linkToBlockIds,
      });
      onSaved();
    } catch (err) {
      setError(String(err));
    }
  }

  async function handleDelete() {
    if (!block) return;
    try {
      await softDeleteBlock(block.id);
      onSaved();
    } catch (err) {
      setError(String(err));
    }
  }

  function toggleLink(id: string) {
    setLinkToBlockIds((prev) =>
      prev.includes(id) ? prev.filter((x) => x !== id) : [...prev, id],
    );
  }

  return (
    <aside className="w-[28rem] shrink-0 overflow-y-auto border-l border-zinc-800 bg-zinc-900/80 p-6 max-h-screen">
      <div className="mb-4 flex items-center justify-between">
        <h2 className="font-medium text-zinc-100">
          {isEdit ? "Edit block" : "New block"}
        </h2>
        <button
          type="button"
          onClick={onClose}
          className="cursor-pointer text-zinc-500 hover:text-zinc-300"
        >
          ✕
        </button>
      </div>

      <p className="mb-4 text-xs text-zinc-500">
        Fill any fields you need. Title only is fine. Add {hypothesisLabel.toLowerCase()}, action, evidence, or conclusion when ready.
      </p>

      {error && (
        <p className="mb-4 rounded-lg border border-red-900/50 bg-red-950/40 px-3 py-2 text-xs text-red-300">
          {error}
        </p>
      )}

      <form onSubmit={handleSubmit} className="space-y-4">
        <label className="block text-sm text-zinc-400">
          Title
          <input
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder={ph.title}
            className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm font-medium text-zinc-100"
          />
        </label>

        <label className="block text-sm text-zinc-400">
          {hypothesisLabel}
          <textarea
            value={hypothesisText}
            onChange={(e) => setHypothesisText(e.target.value)}
            placeholder={ph.hypothesis}
            rows={2}
            className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100"
          />
        </label>

        <label className="block text-sm text-zinc-400">
          Action
          <textarea
            value={actionText}
            onChange={(e) => setActionText(e.target.value)}
            placeholder={ph.action}
            rows={2}
            className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100"
          />
        </label>

        <label className="block text-sm text-zinc-400">
          Evidence
          <textarea
            value={evidenceText}
            onChange={(e) => setEvidenceText(e.target.value)}
            placeholder={ph.evidence}
            rows={3}
            className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100"
          />
        </label>

        <label className="block text-sm text-zinc-400">
          Evidence source (optional URL)
          <input
            value={evidenceSource}
            onChange={(e) => setEvidenceSource(e.target.value)}
            className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100"
          />
        </label>

        <label className="block text-sm text-zinc-400">
          Conclusion
          <textarea
            value={conclusionText}
            onChange={(e) => setConclusionText(e.target.value)}
            placeholder={ph.conclusion}
            rows={2}
            className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100"
          />
        </label>

        {conclusionText.trim() && (
          <div className="space-y-3 rounded-lg border border-zinc-800 p-3">
            <label className="block text-sm text-zinc-400">
              Outcome
              <select
                value={conclusionOutcome}
                onChange={(e) => setConclusionOutcome(e.target.value)}
                className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm"
              >
                <option value="confirmed">Confirmed</option>
                <option value="rejected">Rejected</option>
                <option value="uncertain">Uncertain</option>
                <option value="refined">Refined</option>
              </select>
            </label>
            <label className="block text-sm text-zinc-400">
              Decision tag
              <select
                value={conclusionTag}
                onChange={(e) => setConclusionTag(e.target.value)}
                className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm"
              >
                <option value="none">None</option>
                <option value="pivot">Pivot</option>
                <option value="act">Act</option>
                <option value="ignore">Ignore</option>
                <option value="defer">Defer</option>
              </select>
            </label>
            <label className="block text-sm text-zinc-400">
              Confidence
              <select
                value={confidenceLevel}
                onChange={(e) =>
                  setConfidenceLevel(e.target.value as ConfidenceLevel | "")
                }
                className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm"
              >
                <option value="">—</option>
                {CONFIDENCE_LEVELS.map((c) => (
                  <option key={c.value} value={c.value}>
                    {c.label}
                  </option>
                ))}
              </select>
            </label>
          </div>
        )}

        <div className="grid grid-cols-2 gap-3">
          <label className="block text-sm text-zinc-400">
            Belief state
            <select
              value={beliefState}
              onChange={(e) => setBeliefState(e.target.value as BeliefState)}
              className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm"
            >
              {BELIEF_STATES.map((s) => (
                <option key={s.value} value={s.value}>
                  {s.label}
                </option>
              ))}
            </select>
          </label>
          <label className="block text-sm text-zinc-400">
            System tag
            <select
              value={systemTag}
              onChange={(e) => setSystemTag(e.target.value as BlockSystemTag)}
              className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm"
            >
              {SYSTEM_TAGS.map((t) => (
                <option key={t.value} value={t.value}>
                  {t.label}
                </option>
              ))}
            </select>
          </label>
        </div>

        <label className="block text-sm text-zinc-400">
          Custom tag (optional)
          <input
            value={userTag}
            onChange={(e) => setUserTag(e.target.value)}
            placeholder={ph.userTag}
            className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100"
          />
        </label>

        {pickerBlocks.filter(([id]) => id !== block?.id).length > 0 && (
          <fieldset>
            <legend className="text-sm text-zinc-400">Link to other blocks</legend>
            <p className="mt-1 text-xs text-zinc-500">
              Links are references only. Editing this block does not change linked
              blocks.
            </p>
            <div className="mt-2 max-h-32 space-y-1 overflow-y-auto">
              {pickerBlocks
                .filter(([id]) => id !== block?.id)
                .map(([id, preview]) => (
                  <label
                    key={id}
                    className="flex cursor-pointer items-start gap-2 text-xs text-zinc-300"
                  >
                    <input
                      type="checkbox"
                      checked={linkToBlockIds.includes(id)}
                      onChange={() => toggleLink(id)}
                    />
                    <span className="line-clamp-2">{preview}</span>
                  </label>
                ))}
            </div>
          </fieldset>
        )}

        <div className="flex gap-2">
          <button
            type="submit"
            className="cursor-pointer rounded-lg bg-emerald-600 px-4 py-2 text-sm font-medium text-white hover:bg-emerald-500"
          >
            Save
          </button>
          {isEdit && (
            <button
              type="button"
              onClick={handleDelete}
              className="cursor-pointer rounded-lg border border-red-900/50 px-4 py-2 text-sm text-red-400 hover:bg-red-950/30"
            >
              Delete
            </button>
          )}
        </div>
      </form>
    </aside>
  );
}
