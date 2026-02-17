import { useState } from "react";
import { useTranslation } from "react-i18next";
import { MODEL_GROUPS } from "../types";

interface SettingsPanelProps {
  url: string;
  apiKey: string;
  defaultModel: string;
  onUrlChange: (url: string) => void;
  onApiKeyChange: (key: string) => void;
  onModelChange: (model: string) => void;
  onSyncAll: () => void;
  onRestoreAll: () => void;
  syncing: boolean;
  hasInstalled: boolean;
  apiModels: string[];
  modelsLoading: boolean;
  modelsError: string | null;
}

export function SettingsPanel({
  url,
  apiKey,
  defaultModel,
  onUrlChange,
  onApiKeyChange,
  onModelChange,
  onSyncAll,
  onRestoreAll,
  syncing,
  hasInstalled,
  apiModels,
  modelsLoading,
  modelsError,
}: SettingsPanelProps) {
  const { t } = useTranslation();
  const [showKey, setShowKey] = useState(false);

  return (
    <div className="space-y-3">
      <div className="form-control">
        <label className="label py-1">
          <span className="label-text text-sm font-medium">{t("settings.apiUrl")}</span>
        </label>
        <input
          type="text"
          className="input input-bordered input-sm w-full"
          value={url}
          onChange={(e) => onUrlChange(e.target.value)}
          placeholder="https://free.aipro.love"
        />
      </div>

      <div className="form-control">
        <label className="label py-1">
          <span className="label-text text-sm font-medium">{t("settings.apiKey")}</span>
        </label>
        <div className="join w-full">
          <input
            type={showKey ? "text" : "password"}
            className="input input-bordered input-sm join-item w-full"
            value={apiKey}
            onChange={(e) => onApiKeyChange(e.target.value)}
            placeholder="sk-..."
          />
          <button
            className="btn btn-sm btn-ghost join-item"
            onClick={() => setShowKey(!showKey)}
            aria-label={showKey ? "Hide API key" : "Show API key"}
          >
            {showKey ? "\u{1F648}" : "\u{1F441}"}
          </button>
        </div>
      </div>

      <div className="form-control">
        <label className="label py-1">
          <span className="label-text text-sm font-medium">{t("settings.defaultModel")}</span>
        </label>
        <ModelSelectInline
          value={defaultModel}
          onChange={onModelChange}
          apiModels={apiModels}
          modelsLoading={modelsLoading}
          modelsError={modelsError}
        />
      </div>

      <div className="flex gap-2 pt-1">
        <button
          className="btn btn-primary btn-sm flex-1"
          onClick={onSyncAll}
          disabled={syncing || !apiKey || !hasInstalled}
        >
          {syncing ? (
            <span className="loading loading-spinner loading-xs" />
          ) : null}
          {t("settings.syncAll")}
        </button>
        <button
          className="btn btn-outline btn-sm flex-1"
          onClick={onRestoreAll}
          disabled={syncing}
        >
          {t("settings.restoreAll")}
        </button>
      </div>
    </div>
  );
}

function ModelSelectInline({
  value,
  onChange,
  apiModels,
  modelsLoading,
  modelsError,
}: {
  value: string;
  onChange: (v: string) => void;
  apiModels: string[];
  modelsLoading: boolean;
  modelsError: string | null;
}) {
  const { t } = useTranslation();
  const [custom, setCustom] = useState("");
  const [showCustom, setShowCustom] = useState(false);

  const useApiModels = apiModels.length > 0;
  const allHardcoded = MODEL_GROUPS.flatMap((g) => g.models);
  const isCustom = !allHardcoded.includes(value) && (!useApiModels || !apiModels.includes(value)) && value !== "";

  if (showCustom || isCustom) {
    return (
      <div className="join w-full">
        <input
          type="text"
          className="input input-bordered input-sm join-item w-full"
          value={isCustom ? value : custom}
          onChange={(e) => {
            setCustom(e.target.value);
            onChange(e.target.value);
          }}
          placeholder={t("settings.customModel")}
        />
        <button
          className="btn btn-sm btn-ghost join-item"
          onClick={() => {
            setShowCustom(false);
            if (!custom) onChange("claude-sonnet-4-5");
          }}
        >
          ✕
        </button>
      </div>
    );
  }

  return (
    <div className="join w-full">
      <select
        className="select select-bordered select-sm join-item w-full"
        value={value}
        onChange={(e) => {
          if (e.target.value === "__custom__") {
            setShowCustom(true);
            setCustom("");
          } else {
            onChange(e.target.value);
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
        <span className="btn btn-sm btn-ghost join-item no-animation">
          <span className="loading loading-spinner loading-xs" />
        </span>
      )}
      {modelsError && !modelsLoading && (
        <span
          className="btn btn-sm btn-ghost join-item text-warning no-animation"
          title={modelsError}
        >
          !
        </span>
      )}
    </div>
  );
}
