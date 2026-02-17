import { useTranslation } from "react-i18next";
import { MODEL_GROUPS } from "../types";
import type { CliInfo, CliStatusResult } from "../types";
import { useState } from "react";

interface CliCardProps {
  cli: CliInfo;
  status: CliStatusResult | undefined;
  loading: boolean;
  syncing: boolean;
  restoring: boolean;
  model: string;
  onModelChange: (model: string) => void;
  apiModels: string[];
  modelsLoading: boolean;
  onSync: () => void;
  onRestore: () => void;
  onViewConfig: () => void;
}

export function CliCard({
  cli,
  status,
  loading,
  syncing,
  restoring,
  model,
  onModelChange,
  apiModels,
  modelsLoading,
  onSync,
  onRestore,
  onViewConfig,
}: CliCardProps) {
  const { t } = useTranslation();
  const [showCustom, setShowCustom] = useState(false);
  const [customModel, setCustomModel] = useState("");

  const installed = status?.installed ?? false;
  const version = status?.version;
  const isSynced = status?.is_synced ?? false;
  const hasBackup = status?.has_backup ?? false;
  const currentUrl = status?.current_base_url;
  const syncedCount = status?.synced_count;

  const useApiModels = apiModels.length > 0;
  const allHardcoded = MODEL_GROUPS.flatMap((g) => g.models);
  const isCustomModel = !allHardcoded.includes(model) && (!useApiModels || !apiModels.includes(model)) && model !== "";

  return (
    <div
      className={`card card-compact bg-base-100 border-l-4 ${cli.color} shadow-sm ${
        !installed ? "opacity-50" : ""
      }`}
    >
      <div className="card-body gap-1 p-3">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <span className="text-lg">{cli.icon}</span>
            <span className="font-semibold text-sm">{cli.name}</span>
          </div>
          <div className="flex items-center gap-2">
            {loading ? (
              <span className="badge badge-ghost badge-sm">
                {t("cli.detecting")}
              </span>
            ) : !installed ? (
              <span className="badge badge-ghost badge-sm">
                {t("cli.notDetected")}
              </span>
            ) : (
              <>
                {version && (
                  <span className="badge badge-outline badge-sm">
                    v{version}
                  </span>
                )}
                {isSynced ? (
                  <span className="badge badge-success badge-sm gap-1">
                    {t("cli.synced")}
                  </span>
                ) : (
                  <span className="badge badge-warning badge-sm gap-1">
                    {t("cli.notSynced")}
                  </span>
                )}
              </>
            )}
          </div>
        </div>

        {/* Details (only if installed) */}
        {installed && (
          <>
            <div className="text-xs opacity-60 truncate" title={currentUrl || undefined}>
              {t("cli.currentUrl")}:{" "}
              <span className="font-mono">
                {currentUrl || t("cli.noUrl")}
              </span>
              {syncedCount != null && syncedCount > 0 && (
                <span className="ml-2">
                  ({t("cli.syncedModels", { count: syncedCount })})
                </span>
              )}
            </div>

            {/* Model selector */}
            <div className="flex items-center gap-2 mt-1">
              <span className="text-xs opacity-60 shrink-0">
                {t("cli.model")}:
              </span>
              {showCustom || isCustomModel ? (
                <div className="join flex-1">
                  <input
                    type="text"
                    className="input input-bordered input-xs join-item w-full"
                    value={isCustomModel ? model : customModel}
                    onChange={(e) => {
                      setCustomModel(e.target.value);
                      onModelChange(e.target.value);
                    }}
                    placeholder="model-id"
                  />
                  <button
                    className="btn btn-xs btn-ghost join-item"
                    onClick={() => {
                      setShowCustom(false);
                      setCustomModel("");
                      onModelChange("claude-sonnet-4-5");
                    }}
                  >
                    ✕
                  </button>
                </div>
              ) : (
                <div className="join flex-1">
                  <select
                    className="select select-bordered select-xs join-item w-full"
                    value={model}
                    onChange={(e) => {
                      if (e.target.value === "__custom__") {
                        setShowCustom(true);
                        setCustomModel("");
                      } else {
                        onModelChange(e.target.value);
                      }
                    }}
                  >
                    {useApiModels ? (
                      <optgroup label={t("settings.apiModelsGroup")}>
                        {apiModels.map((m) => (
                          <option key={m} value={m}>
                            {m}
                          </option>
                        ))}
                      </optgroup>
                    ) : (
                      MODEL_GROUPS.map((group) => (
                        <optgroup key={group.label} label={group.label}>
                          {group.models.map((m) => (
                            <option key={m} value={m}>
                              {m}
                            </option>
                          ))}
                        </optgroup>
                      ))
                    )}
                    <optgroup label="---">
                      <option value="__custom__">{t("settings.customModel")}</option>
                    </optgroup>
                  </select>
                  {modelsLoading && (
                    <span className="btn btn-xs btn-ghost join-item no-animation">
                      <span className="loading loading-spinner loading-xs" />
                    </span>
                  )}
                </div>
              )}
            </div>

            {/* Actions */}
            <div className="flex gap-1.5 mt-1">
              <button
                className="btn btn-primary btn-xs flex-1"
                onClick={onSync}
                disabled={syncing}
              >
                {syncing ? (
                  <span className="loading loading-spinner loading-xs" />
                ) : null}
                {t("cli.sync")}
              </button>
              <button
                className="btn btn-outline btn-xs flex-1"
                onClick={onRestore}
                disabled={restoring || !hasBackup}
              >
                {restoring ? (
                  <span className="loading loading-spinner loading-xs" />
                ) : null}
                {t("cli.restore")}
              </button>
              <button
                className="btn btn-ghost btn-xs flex-1"
                onClick={onViewConfig}
              >
                {t("cli.viewConfig")}
              </button>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
