import { useState } from "react";
import { useTranslation } from "react-i18next";

interface ModelSelectorProps {
  value: string;
  onChange: (model: string) => void;
  apiModels: string[];
  modelsLoading: boolean;
  modelsError?: string | null;
  size?: "xs" | "sm";
}

export function ModelSelector({
  value,
  onChange,
  apiModels,
  modelsLoading,
  modelsError,
  size = "xs",
}: ModelSelectorProps) {
  const { t } = useTranslation();
  const [showCustom, setShowCustom] = useState(false);
  const [customModel, setCustomModel] = useState("");

  const hasApiModels = apiModels.length > 0;
  const isInList = value !== "" && apiModels.includes(value);

  const inputClass = size === "sm" ? "input-sm" : "input-xs";
  const selectClass = size === "sm" ? "select-sm" : "select-xs";
  const btnClass = size === "sm" ? "btn-sm" : "btn-xs";

  // No API models available — show text input only
  if (!hasApiModels || showCustom) {
    return (
      <div className="join w-full flex-1">
        <input
          type="text"
          className={`input input-bordered ${inputClass} join-item w-full`}
          value={!isInList ? value : customModel}
          onChange={(e) => {
            setCustomModel(e.target.value);
            onChange(e.target.value);
          }}
          placeholder={t("settings.customModel")}
        />
        {hasApiModels && (
          <button
            className={`btn ${btnClass} btn-ghost join-item`}
            onClick={() => {
              setShowCustom(false);
              setCustomModel("");
              // Restore the previously selected list value (don't clobber user's choice)
              if (!isInList && apiModels.length > 0) onChange(apiModels[0]);
            }}
          >
            ✕
          </button>
        )}
        {modelsLoading && (
          <span className={`btn ${btnClass} btn-ghost join-item no-animation`}>
            <span className="loading loading-spinner loading-xs" />
          </span>
        )}
      </div>
    );
  }

  return (
    <div className="join w-full flex-1">
      <select
        className={`select select-bordered ${selectClass} join-item w-full`}
        value={value}
        onChange={(e) => {
          if (e.target.value === "__custom__") {
            setShowCustom(true);
            setCustomModel("");
          } else {
            onChange(e.target.value);
          }
        }}
      >
        {!isInList && value && (
          <optgroup label={t("settings.currentModel")}>
            <option value={value}>{value}</option>
          </optgroup>
        )}
        <optgroup label={t("settings.apiModelsGroup")}>
          {apiModels.map((m) => (
            <option key={m} value={m}>
              {m}
            </option>
          ))}
        </optgroup>
        <optgroup label="---">
          <option value="__custom__">{t("settings.customModel")}</option>
        </optgroup>
      </select>
      {modelsLoading && (
        <span className={`btn ${btnClass} btn-ghost join-item no-animation`}>
          <span className="loading loading-spinner loading-xs" />
        </span>
      )}
      {modelsError && !modelsLoading && (
        <span
          className={`btn ${btnClass} btn-ghost join-item text-warning no-animation`}
          title={modelsError}
        >
          !
        </span>
      )}
    </div>
  );
}
