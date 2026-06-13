import { useEffect, useState } from "react";
import {
  createAction,
  createEvidence,
  createHypothesis,
  editHypothesis,
  listPickerNodes,
  saveConclusion,
  softDeleteNode,
} from "../api";
import {
  NODE_TYPES,
  PLACEHOLDERS,
  nodeTypeLabel,
  type NodeType,
  type PickerNode,
  type TimelineEntry,
  type Workspace,
} from "../types";

interface Props {
  workspace: Workspace;
  entry: TimelineEntry | null;
  createType: NodeType | null;
  onClose: () => void;
  onSaved: () => void;
}

export default function NodePanel({
  workspace,
  entry,
  createType,
  onClose,
  onSaved,
}: Props) {
  const isCreate = !!createType;
  const nodeType = createType ?? entry?.node_type ?? null;

  const [step, setStep] = useState<"pick" | "form">(
    isCreate && !createType ? "pick" : "form",
  );
  const [pickedType, setPickedType] = useState<NodeType | null>(createType);
  const [text, setText] = useState(entry?.text ?? "");
  const [source, setSource] = useState(entry?.source ?? "");
  const [outcome, setOutcome] = useState(entry?.outcome ?? "uncertain");
  const [tag, setTag] = useState(entry?.tag ?? "none");
  const [confidence, setConfidence] = useState("");
  const [hypothesisIds, setHypothesisIds] = useState<string[]>([]);
  const [evidenceIds, setEvidenceIds] = useState<string[]>([]);
  const [hypotheses, setHypotheses] = useState<PickerNode[]>([]);
  const [evidenceList, setEvidenceList] = useState<PickerNode[]>([]);
  const [error, setError] = useState("");

  const activeType = pickedType ?? nodeType;
  const template = workspace.template;

  useEffect(() => {
    if (activeType === "conclusion") {
      listPickerNodes(workspace.id, "hypothesis").then(setHypotheses);
      listPickerNodes(workspace.id, "evidence").then(setEvidenceList);
    }
  }, [activeType, workspace.id]);

  const placeholder =
    activeType && template
      ? PLACEHOLDERS[template][activeType]
      : "Enter reasoning node…";

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!activeType) return;
    setError("");
    try {
      if (entry && entry.node_type === "hypothesis") {
        await editHypothesis(entry.id, text);
        onSaved();
        return;
      }

      if (activeType === "hypothesis") {
        const h = await createHypothesis(workspace.id, text);
        await onSavedWithOptionalLinks("hypothesis", (h as { id: string }).id);
        return;
      }
      if (activeType === "action") {
        const a = await createAction(workspace.id, text);
        await onSavedWithOptionalLinks("action", (a as { id: string }).id);
        return;
      }
      if (activeType === "evidence") {
        const ev = await createEvidence(workspace.id, text, source || undefined);
        await onSavedWithOptionalLinks("evidence", (ev as { id: string }).id);
        return;
      }
      if (activeType === "conclusion") {
        await saveConclusion({
          workspaceId: workspace.id,
          text,
          outcome,
          tag,
          confidence: confidence ? Number(confidence) : undefined,
          hypothesisIds,
          evidenceIds,
        });
        onSaved();
      }
    } catch (err) {
      setError(String(err));
    }
  }

  async function onSavedWithOptionalLinks(
    _fromType: NodeType,
    _fromId: string,
  ) {
    onSaved();
  }

  async function handleDelete() {
    if (!entry || !activeType) return;
    try {
      await softDeleteNode(activeType, entry.id);
      onSaved();
    } catch (err) {
      setError(String(err));
    }
  }

  function toggleId(list: string[], id: string): string[] {
    return list.includes(id) ? list.filter((x) => x !== id) : [...list, id];
  }

  if (step === "pick" && isCreate) {
    return (
      <aside className="w-96 shrink-0 border-l border-zinc-800 bg-zinc-900/80 p-6">
        <div className="mb-4 flex items-center justify-between">
          <h2 className="font-medium text-zinc-100">Add node</h2>
          <button type="button" onClick={onClose} className="text-zinc-500 hover:text-zinc-300">
            ✕
          </button>
        </div>
        <p className="mb-4 text-sm text-zinc-400">Pick a type first — no blank notes.</p>
        <div className="space-y-2">
          {NODE_TYPES.map((t) => (
            <button
              key={t.value}
              type="button"
              onClick={() => {
                setPickedType(t.value);
                setStep("form");
              }}
              className="block w-full rounded-lg border border-zinc-700 px-4 py-3 text-left text-sm text-zinc-200 hover:border-zinc-500"
            >
              {t.label}
            </button>
          ))}
        </div>
      </aside>
    );
  }

  return (
    <aside className="w-96 shrink-0 border-l border-zinc-800 bg-zinc-900/80 p-6 overflow-y-auto max-h-screen">
      <div className="mb-4 flex items-center justify-between">
        <h2 className="font-medium text-zinc-100">
          {entry
            ? "Node detail"
            : activeType
              ? `New ${nodeTypeLabel(activeType)}`
              : "New node"}
        </h2>
        <button type="button" onClick={onClose} className="text-zinc-500 hover:text-zinc-300">
          ✕
        </button>
      </div>

      {entry?.unlinked && (
        <p className="mb-4 rounded-lg border border-orange-900/40 bg-orange-950/30 px-3 py-2 text-xs text-orange-200">
          This does not yet participate in the reasoning graph.
        </p>
      )}

      {error && (
        <p className="mb-4 rounded-lg border border-red-900/50 bg-red-950/40 px-3 py-2 text-xs text-red-300">
          {error}
        </p>
      )}

      <form onSubmit={handleSubmit} className="space-y-4">
        <label className="block text-sm text-zinc-400">
          {activeType ? nodeTypeLabel(activeType) : "Text"}
          <textarea
            required
            value={text}
            onChange={(e) => setText(e.target.value)}
            placeholder={placeholder}
            rows={5}
            disabled={!!entry && entry.node_type !== "hypothesis"}
            className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 disabled:opacity-60"
          />
        </label>

        {activeType === "evidence" && !entry && (
          <label className="block text-sm text-zinc-400">
            Source (optional URL)
            <input
              value={source}
              onChange={(e) => setSource(e.target.value)}
              className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100"
            />
          </label>
        )}

        {activeType === "conclusion" && !entry && (
          <>
            <label className="block text-sm text-zinc-400">
              Outcome
              <select
                value={outcome}
                onChange={(e) => setOutcome(e.target.value)}
                className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm"
              >
                <option value="confirmed">confirmed</option>
                <option value="rejected">rejected</option>
                <option value="uncertain">uncertain</option>
                <option value="refined">refined</option>
              </select>
            </label>
            <label className="block text-sm text-zinc-400">
              Tag
              <select
                value={tag}
                onChange={(e) => setTag(e.target.value)}
                className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm"
              >
                <option value="none">none</option>
                <option value="pivot">pivot</option>
                <option value="act">act</option>
                <option value="ignore">ignore</option>
                <option value="defer">defer</option>
              </select>
            </label>
            <label className="block text-sm text-zinc-400">
              Confidence (0–1, optional)
              <input
                type="number"
                min={0}
                max={1}
                step={0.1}
                value={confidence}
                onChange={(e) => setConfidence(e.target.value)}
                className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm"
              />
            </label>
            <fieldset>
              <legend className="text-sm text-zinc-400">
                Linked hypotheses (required)
              </legend>
              <div className="mt-2 max-h-32 space-y-1 overflow-y-auto">
                {hypotheses.map((h) => (
                  <label key={h.id} className="flex items-start gap-2 text-xs text-zinc-300">
                    <input
                      type="checkbox"
                      checked={hypothesisIds.includes(h.id)}
                      onChange={() => setHypothesisIds(toggleId(hypothesisIds, h.id))}
                    />
                    <span className="line-clamp-2">{h.text}</span>
                  </label>
                ))}
              </div>
            </fieldset>
            <fieldset>
              <legend className="text-sm text-zinc-400">
                Linked evidence (required)
              </legend>
              <div className="mt-2 max-h-32 space-y-1 overflow-y-auto">
                {evidenceList.map((ev) => (
                  <label key={ev.id} className="flex items-start gap-2 text-xs text-zinc-300">
                    <input
                      type="checkbox"
                      checked={evidenceIds.includes(ev.id)}
                      onChange={() => setEvidenceIds(toggleId(evidenceIds, ev.id))}
                    />
                    <span className="line-clamp-2">{ev.text}</span>
                  </label>
                ))}
              </div>
            </fieldset>
          </>
        )}

        <div className="flex gap-2">
          <button
            type="submit"
            className="cursor-pointer rounded-lg bg-emerald-600 px-4 py-2 text-sm font-medium text-white hover:bg-emerald-500"
          >
            Save
          </button>
          {entry && (
            <button
              type="button"
              onClick={handleDelete}
              className="rounded-lg border border-red-900/50 px-4 py-2 text-sm text-red-400 hover:bg-red-950/30"
            >
              Soft delete
            </button>
          )}
        </div>
      </form>
    </aside>
  );
}
