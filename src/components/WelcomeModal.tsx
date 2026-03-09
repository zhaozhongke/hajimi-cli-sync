import { useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { Check, Eye, EyeOff } from "lucide-react";

interface WelcomeModalProps {
  defaultUrl: string;
  onComplete: (url: string, apiKey: string, providerName: string) => void;
  onSkip: () => void;
}

export function WelcomeModal({ defaultUrl, onComplete, onSkip }: WelcomeModalProps) {
  const { t } = useTranslation();
  const [step, setStep] = useState(1);
  const [url, setUrl] = useState(defaultUrl);
  const [apiKey, setApiKey] = useState("");
  const [providerName, setProviderName] = useState("");
  const [showKey, setShowKey] = useState(false);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<"success" | "error" | null>(null);

  const handleTestConnection = async () => {
    if (!url.trim() || !apiKey.trim()) return;
    setTesting(true);
    setTestResult(null);
    try {
      await invoke("test_connection", { url, apiKey });
      setTestResult("success");
    } catch {
      setTestResult("error");
    } finally {
      setTesting(false);
    }
  };

  const stepDot = (s: number) => {
    const isDone = step > s;
    const isActive = step === s;
    return (
      <div
        className={`w-6 h-6 rounded-full flex items-center justify-center text-xs font-bold transition-all ${
          isDone
            ? "bg-success text-success-content"
            : isActive
            ? "bg-primary text-primary-content"
            : "bg-base-300 text-base-content/40"
        }`}
      >
        {isDone ? <Check className="w-3.5 h-3.5" /> : s}
      </div>
    );
  };

  return (
    <dialog className="modal modal-open">
      <div className="modal-box max-w-md">
        {/* Step indicator */}
        <div className="flex items-center justify-center gap-3 mb-6">
          {stepDot(1)}
          <div className={`h-0.5 w-8 transition-colors ${step > 1 ? "bg-success" : "bg-base-300"}`} />
          {stepDot(2)}
          <div className={`h-0.5 w-8 transition-colors ${step > 2 ? "bg-success" : "bg-base-300"}`} />
          {stepDot(3)}
        </div>

        {/* Step 1: Welcome */}
        {step === 1 && (
          <div className="text-center space-y-4">
            <svg
              width="64" height="64" viewBox="0 0 512 512"
              xmlns="http://www.w3.org/2000/svg"
              className="mx-auto rounded-2xl shadow-lg"
              style={{ filter: "drop-shadow(0 4px 12px #6366f140)" }}
            >
              <defs>
                <linearGradient id="welcome-logo-grad" x1="0%" y1="0%" x2="100%" y2="100%">
                  <stop offset="0%" stopColor="#6366f1" />
                  <stop offset="50%" stopColor="#8b5cf6" />
                  <stop offset="100%" stopColor="#06b6d4" />
                </linearGradient>
              </defs>
              <rect width="512" height="512" rx="108" ry="108" fill="url(#welcome-logo-grad)" />
              <g transform="translate(256,256) scale(13) translate(-12,-12)" fill="white" stroke="none">
                <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2" />
              </g>
            </svg>
            <h2 className="text-xl font-bold">{t("welcome.title")}</h2>
            <p className="text-sm opacity-60">{t("welcome.subtitle")}</p>
            <button
              className="btn btn-primary btn-wide"
              onClick={() => setStep(2)}
            >
              {t("welcome.getStarted")}
            </button>
            <div>
              <button
                className="btn btn-ghost btn-sm opacity-40 hover:opacity-70"
                onClick={onSkip}
              >
                {t("welcome.skip")}
              </button>
            </div>
          </div>
        )}

        {/* Step 2: Configure */}
        {step === 2 && (
          <div className="space-y-4">
            <h2 className="text-lg font-bold text-center">{t("welcome.configTitle")}</h2>

            <div className="form-control">
              <label className="label py-1">
                <span className="label-text text-xs">{t("welcome.urlLabel")}</span>
              </label>
              <input
                type="text"
                className="input input-bordered input-sm w-full"
                value={url}
                onChange={(e) => setUrl(e.target.value)}
                placeholder="https://..."
              />
            </div>

            <div className="form-control">
              <label className="label py-1">
                <span className="label-text text-xs">{t("welcome.apiKeyLabel")}</span>
              </label>
              <div className="relative">
                <input
                  type={showKey ? "text" : "password"}
                  className="input input-bordered input-sm w-full pr-10"
                  value={apiKey}
                  onChange={(e) => setApiKey(e.target.value)}
                  placeholder="sk-..."
                />
                <button
                  type="button"
                  className="absolute right-2 top-1/2 -translate-y-1/2 btn btn-ghost btn-xs btn-square"
                  onClick={() => setShowKey(!showKey)}
                >
                  {showKey ? <EyeOff className="w-3.5 h-3.5" /> : <Eye className="w-3.5 h-3.5" />}
                </button>
              </div>
            </div>

            <div className="form-control">
              <label className="label py-1">
                <span className="label-text text-xs">{t("welcome.providerNameLabel")}</span>
              </label>
              <input
                type="text"
                className="input input-bordered input-sm w-full"
                value={providerName}
                onChange={(e) => setProviderName(e.target.value)}
                placeholder={t("welcome.providerNameDefault")}
              />
            </div>

            {/* Test connection */}
            <div className="flex items-center gap-2">
              <button
                className="btn btn-outline btn-sm"
                onClick={handleTestConnection}
                disabled={testing || !url.trim() || !apiKey.trim()}
              >
                {testing ? (
                  <span className="loading loading-spinner loading-xs" />
                ) : null}
                {t("welcome.testConnection")}
              </button>
              {testResult === "success" && (
                <span className="text-success text-xs flex items-center gap-1">
                  <Check className="w-3.5 h-3.5" />
                  {t("welcome.testSuccess")}
                </span>
              )}
              {testResult === "error" && (
                <span className="text-error text-xs">{t("welcome.testFailed")}</span>
              )}
            </div>

            <div className="flex justify-between items-center pt-2">
              <button
                className="btn btn-ghost btn-sm opacity-40 hover:opacity-70"
                onClick={onSkip}
              >
                {t("welcome.skip")}
              </button>
              <button
                className="btn btn-primary btn-sm"
                onClick={() => setStep(3)}
                disabled={!url.trim() || !apiKey.trim()}
              >
                {t("welcome.next")}
              </button>
            </div>
          </div>
        )}

        {/* Step 3: Done */}
        {step === 3 && (
          <div className="text-center space-y-4">
            <div className="w-16 h-16 rounded-full bg-success/20 flex items-center justify-center mx-auto">
              <Check className="w-8 h-8 text-success" />
            </div>
            <h2 className="text-xl font-bold">{t("welcome.doneTitle")}</h2>
            <p className="text-sm opacity-60">{t("welcome.doneSubtitle")}</p>
            <button
              className="btn btn-primary btn-wide"
              onClick={() =>
                onComplete(url, apiKey, providerName.trim() || t("welcome.providerNameDefault"))
              }
            >
              {t("welcome.enterApp")}
            </button>
          </div>
        )}
      </div>
      {/* Non-dismissible backdrop */}
      <div className="modal-backdrop" />
    </dialog>
  );
}
