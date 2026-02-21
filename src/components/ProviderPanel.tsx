import { useState, useCallback, useRef } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { Plus, Trash2, Check, Edit2, ChevronUp, ChevronDown, X, AlertTriangle } from "lucide-react";
import type { ProviderRecord, SwitchResult } from "../types";
import {
  saveProvider,
  deleteProvider,
  switchProvider,
  reorderProviders,
} from "../hooks/useProviders";

function maskKey(key: string): string {
  if (!key) return "—";
  if (key.length <= 8) return "••••••••";
  return key.slice(0, 4) + "••••" + key.slice(-4);
}

// ── Form component (isolated so state never leaks between new/edit) ──────────

interface FormState {
  id: string;
  name: string;
  url: string;
  api_key: string;
  default_model: string;
  notes: string;
}

interface ProviderFormProps {
  initial: FormState;
  isNew: boolean;
  onSave: (f: FormState) => Promise<void>;
  onCancel: () => void;
}

function ProviderForm({ initial, isNew, onSave, onCancel }: ProviderFormProps) {
  const { t } = useTranslation();
  const [form, setForm] = useState<FormState>(initial);
  const [showKey, setShowKey] = useState(false);
  const [saving, setSaving] = useState(false);

  const field = (key: keyof FormState) => ({
    value: form[key] as string,
    onChange: (e: React.ChangeEvent<HTMLInputElement>) =>
      setForm((f) => ({ ...f, [key]: e.target.value })),
  });

  const handleSave = async () => {
    if (!form.name.trim()) { toast.error(t("provider.nameRequired")); return; }
    if (!form.url.trim()) { toast.error(t("provider.urlRequired")); return; }
    if (!form.api_key.trim()) { toast.error(t("provider.apiKeyRequired")); return; }
    setSaving(true);
    try {
      await onSave(form);
    } catch (e) {
      toast.error(String(e), { duration: 5000 });
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="card glass-card shadow-sm mt-2 border border-primary/20">
      <div className="card-body p-3 gap-2">
        <div className="flex items-center justify-between mb-0.5">
          <span className="text-xs font-semibold opacity-70">
            {isNew ? t("provider.newTitle") : t("provider.editTitle")}
          </span>
          <button className="btn btn-ghost btn-xs btn-square" onClick={onCancel}>
            <X className="w-3 h-3" />
          </button>
        </div>

        <input
          className="input input-bordered input-sm w-full"
          placeholder={t("provider.namePlaceholder")}
          autoFocus
          {...field("name")}
          onKeyDown={(e) => e.key === "Enter" && handleSave()}
        />
        <input
          className="input input-bordered input-sm w-full"
          placeholder={t("provider.urlPlaceholder")}
          {...field("url")}
          onKeyDown={(e) => e.key === "Enter" && handleSave()}
        />
        <div className="relative">
          <input
            className="input input-bordered input-sm w-full pr-16"
            placeholder={t("provider.apiKeyPlaceholder")}
            type={showKey ? "text" : "password"}
            {...field("api_key")}
            onKeyDown={(e) => e.key === "Enter" && handleSave()}
          />
          <button
            className="btn btn-ghost btn-xs absolute right-1 top-1/2 -translate-y-1/2 opacity-50 hover:opacity-100"
            onClick={() => setShowKey((v) => !v)}
            tabIndex={-1}
          >
            {showKey ? t("provider.hideKey") : t("provider.showKey")}
          </button>
        </div>
        <input
          className="input input-bordered input-sm w-full"
          placeholder={t("provider.defaultModelPlaceholder")}
          {...field("default_model")}
          onKeyDown={(e) => e.key === "Enter" && handleSave()}
        />
        <input
          className="input input-bordered input-sm w-full"
          placeholder={t("provider.notesPlaceholder")}
          {...field("notes")}
          onKeyDown={(e) => e.key === "Enter" && handleSave()}
        />

        <div className="flex gap-2 justify-end pt-1">
          <button className="btn btn-ghost btn-xs" onClick={onCancel}>
            {t("config.cancel")}
          </button>
          <button
            className="btn btn-primary btn-xs"
            onClick={handleSave}
            disabled={saving}
          >
            {saving
              ? <span className="loading loading-spinner loading-xs" />
              : t("config.save")}
          </button>
        </div>
      </div>
    </div>
  );
}

// ── Confirm-delete inline prompt ─────────────────────────────────────────────

interface DeleteConfirmProps {
  name: string;
  onConfirm: () => void;
  onCancel: () => void;
}

function DeleteConfirm({ name, onConfirm, onCancel }: DeleteConfirmProps) {
  const { t } = useTranslation();
  return (
    <div className="flex items-center gap-2 px-2 py-1.5 rounded-lg bg-error/10 border border-error/20 mt-1 animate-in fade-in slide-in-from-top-1 duration-150">
      <AlertTriangle className="w-3.5 h-3.5 text-error shrink-0" />
      <span className="text-xs flex-1 truncate">
        {t("provider.deleteConfirm", { name })}
      </span>
      <button className="btn btn-ghost btn-xs" onClick={onCancel}>
        {t("confirm.cancel")}
      </button>
      <button className="btn btn-error btn-xs" onClick={onConfirm}>
        {t("provider.deleteConfirmBtn")}
      </button>
    </div>
  );
}

// ── Main panel ───────────────────────────────────────────────────────────────

interface ProviderPanelProps {
  providers: ProviderRecord[];
  onProvidersChange: () => Promise<unknown>;
  /** Called after a successful switch with the newly-current provider. */
  onSwitched: (provider: ProviderRecord) => void;
  isSwitching: boolean;
  setIsSwitching: (v: boolean) => void;
}

export function ProviderPanel({
  providers,
  onProvidersChange,
  onSwitched,
  isSwitching,
  setIsSwitching,
}: ProviderPanelProps) {
  const { t } = useTranslation();

  // Which provider (if any) is showing its edit form
  const [editingId, setEditingId] = useState<string | null>(null);
  // New-provider form open?
  const [showNew, setShowNew] = useState(false);
  // Pending-delete id (shows inline confirm)
  const [deletingId, setDeletingId] = useState<string | null>(null);
  // Which provider is currently mid-switch (for per-row spinner)
  const [switchingId, setSwitchingId] = useState<string | null>(null);

  // Stable ref for new-form id so it doesn't regenerate on re-render
  const newIdRef = useRef<string>("");

  const openNew = useCallback(() => {
    newIdRef.current = crypto.randomUUID();
    setEditingId(null);
    setDeletingId(null);
    setShowNew(true);
  }, []);

  const closeNew = useCallback(() => setShowNew(false), []);

  const openEdit = useCallback((id: string) => {
    setShowNew(false);
    setDeletingId(null);
    setEditingId((prev) => (prev === id ? null : id));
  }, []);

  const closeEdit = useCallback(() => setEditingId(null), []);

  // ── Save (new or edit) ────────────────────────────────────────────────────

  const handleSave = useCallback(
    async (form: FormState, isNew: boolean) => {
      const existing = providers.find((p) => p.id === form.id);
      const record: ProviderRecord = {
        id: form.id,
        name: form.name.trim(),
        url: form.url.trim(),
        api_key: form.api_key.trim(),
        default_model: form.default_model.trim(),
        per_cli_models: existing?.per_cli_models ?? "{}",
        is_current: existing?.is_current ?? false,
        sort_index: existing?.sort_index ?? null,
        notes: form.notes.trim() || null,
        // Unix seconds — consistent with Rust's i64 created_at column.
        created_at: existing?.created_at ?? Math.floor(Date.now() / 1000),
      };
      await saveProvider(record);
      await onProvidersChange();
      if (isNew) setShowNew(false);
      else setEditingId(null);
      toast.success(t("provider.saved"));
    },
    [providers, onProvidersChange, t]
  );

  // ── Switch ────────────────────────────────────────────────────────────────

  const handleSwitch = useCallback(
    async (p: ProviderRecord) => {
      if (p.is_current || isSwitching) return;
      setSwitchingId(p.id);
      setIsSwitching(true);
      try {
        const result: SwitchResult = await switchProvider(p.id);
        // Reload providers — App's useEffect[currentProvider] drives url/apiKey/model.
        await onProvidersChange();

        if (result.success) {
          toast.success(t("provider.switched", { name: p.name }));
        } else {
          const errApps = result.errors.map((e) => e.app).join(", ");
          toast.warning(t("provider.switchedWithErrors", { apps: errApps }));
        }
      } catch (e) {
        toast.error(String(e), { duration: 5000 });
      } finally {
        setSwitchingId(null);
        setIsSwitching(false);
      }
    },
    [isSwitching, onProvidersChange, setIsSwitching, t]
  );

  // ── Delete ────────────────────────────────────────────────────────────────

  const handleDeleteConfirmed = useCallback(
    async (id: string) => {
      setDeletingId(null);
      try {
        await deleteProvider(id);
        await onProvidersChange();
        toast.success(t("provider.deleted"));
      } catch (e) {
        toast.error(String(e), { duration: 5000 });
      }
    },
    [onProvidersChange, t]
  );

  // ── Reorder ───────────────────────────────────────────────────────────────

  const handleReorder = useCallback(
    async (idx: number, direction: -1 | 1) => {
      const target = idx + direction;
      if (target < 0 || target >= providers.length) return;
      const ids = providers.map((p) => p.id);
      [ids[idx], ids[target]] = [ids[target], ids[idx]];
      try {
        await reorderProviders(ids);
        await onProvidersChange();
      } catch (e) {
        toast.error(String(e), { duration: 5000 });
      }
    },
    [providers, onProvidersChange]
  );

  // ── Render ────────────────────────────────────────────────────────────────

  return (
    <div className="space-y-2">
      {/* Header */}
      <div className="flex items-center justify-between">
        <span className="text-xs font-semibold opacity-60">
          {t("provider.title")}
        </span>
        <button
          className="btn btn-ghost btn-xs gap-1 opacity-60 hover:opacity-100 transition-opacity"
          onClick={openNew}
          disabled={showNew || isSwitching}
        >
          <Plus className="w-3 h-3" />
          {t("provider.add")}
        </button>
      </div>

      {/* New-provider form */}
      {showNew && (
        <ProviderForm
          key={newIdRef.current}
          initial={{
            id: newIdRef.current,
            name: "",
            url: "",
            api_key: "",
            default_model: "",
            notes: "",
          }}
          isNew
          onSave={(f) => handleSave(f, true)}
          onCancel={closeNew}
        />
      )}

      {/* Provider list */}
      <div className="space-y-1.5">
        {providers.map((p, idx) => {
          const isThisSwitching = switchingId === p.id;
          return (
            <div key={p.id}>
              {/* Row */}
              <div
                className={`flex items-center gap-2 p-2 rounded-lg border transition-all duration-150 ${
                  p.is_current
                    ? "border-primary/50 bg-primary/8 shadow-sm"
                    : "border-base-300/50 bg-base-100/30 hover:bg-base-200/30"
                }`}
              >
                {/* Current check */}
                <div className="shrink-0 w-4">
                  {p.is_current && (
                    <Check className="w-3.5 h-3.5 text-primary" />
                  )}
                </div>

                {/* Info */}
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-1.5">
                    <span className="text-xs font-semibold truncate">
                      {p.name}
                    </span>
                    {p.is_current && (
                      <span className="badge badge-primary badge-xs shrink-0">
                        {t("provider.current")}
                      </span>
                    )}
                  </div>
                  <div className="text-[10px] opacity-40 truncate font-mono mt-0.5">
                    {p.url}
                    <span className="mx-1">·</span>
                    {maskKey(p.api_key)}
                    {p.default_model && (
                      <>
                        <span className="mx-1">·</span>
                        {p.default_model}
                      </>
                    )}
                  </div>
                </div>

                {/* Reorder (disabled while switching) */}
                <div className="flex flex-col shrink-0 gap-0">
                  <button
                    className="btn btn-ghost btn-xs btn-square h-3.5 min-h-0 opacity-25 hover:opacity-70 disabled:opacity-10"
                    onClick={() => handleReorder(idx, -1)}
                    disabled={idx === 0 || !!isSwitching}
                    tabIndex={-1}
                  >
                    <ChevronUp className="w-2.5 h-2.5" />
                  </button>
                  <button
                    className="btn btn-ghost btn-xs btn-square h-3.5 min-h-0 opacity-25 hover:opacity-70 disabled:opacity-10"
                    onClick={() => handleReorder(idx, 1)}
                    disabled={idx === providers.length - 1 || !!isSwitching}
                    tabIndex={-1}
                  >
                    <ChevronDown className="w-2.5 h-2.5" />
                  </button>
                </div>

                {/* Actions */}
                <div className="flex items-center gap-0.5 shrink-0">
                  {!p.is_current && (
                    <button
                      className="btn btn-primary btn-xs min-w-[3.5rem]"
                      onClick={() => handleSwitch(p)}
                      disabled={!!isSwitching}
                    >
                      {isThisSwitching ? (
                        <span className="loading loading-spinner loading-xs" />
                      ) : (
                        t("provider.switch")
                      )}
                    </button>
                  )}
                  <button
                    className={`btn btn-ghost btn-xs btn-square opacity-50 hover:opacity-100 transition-opacity ${
                      editingId === p.id ? "bg-base-200" : ""
                    }`}
                    onClick={() =>
                      editingId === p.id ? closeEdit() : openEdit(p.id)
                    }
                    disabled={!!isSwitching}
                  >
                    <Edit2 className="w-3 h-3" />
                  </button>
                  <button
                    className="btn btn-ghost btn-xs btn-square opacity-50 hover:opacity-100 hover:text-error transition-all disabled:opacity-20"
                    onClick={() =>
                      p.is_current
                        ? toast.error(t("provider.cannotDeleteCurrent"))
                        : setDeletingId((prev) => (prev === p.id ? null : p.id))
                    }
                    disabled={!!isSwitching}
                  >
                    <Trash2 className="w-3 h-3" />
                  </button>
                </div>
              </div>

              {/* Inline delete confirm */}
              {deletingId === p.id && (
                <DeleteConfirm
                  name={p.name}
                  onConfirm={() => handleDeleteConfirmed(p.id)}
                  onCancel={() => setDeletingId(null)}
                />
              )}

              {/* Inline edit form — isolated component, no shared state */}
              {editingId === p.id && (
                <ProviderForm
                  key={p.id}
                  initial={{
                    id: p.id,
                    name: p.name,
                    url: p.url,
                    api_key: p.api_key,
                    default_model: p.default_model,
                    notes: p.notes ?? "",
                  }}
                  isNew={false}
                  onSave={(f) => handleSave(f, false)}
                  onCancel={closeEdit}
                />
              )}
            </div>
          );
        })}
      </div>

      {/* Empty state */}
      {providers.length === 0 && !showNew && (
        <div className="text-center py-4 opacity-40">
          <p className="text-xs">{t("provider.empty")}</p>
        </div>
      )}
    </div>
  );
}
