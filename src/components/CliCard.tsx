import { useTranslation } from "react-i18next";
import {
  Terminal, Code, Sparkles, FileCode, Bot, MousePointer,
  MessageSquare, Cherry, Cpu, FileText, Rabbit, Ruler,
  Beer, Brain, Zap, Waves, Check, CircleDot,
  type LucideIcon,
} from "lucide-react";
import type { CliInfo, CliStatusResult } from "../types";
import { ModelSelector } from "./ModelSelector";

const iconMap: Record<string, LucideIcon> = {
  terminal: Terminal,
  code: Code,
  sparkles: Sparkles,
  "file-code": FileCode,
  bot: Bot,
  "mouse-pointer": MousePointer,
  "message-square": MessageSquare,
  cherry: Cherry,
  cpu: Cpu,
  "file-text": FileText,
  rabbit: Rabbit,
  ruler: Ruler,
  beer: Beer,
  brain: Brain,
  zap: Zap,
  waves: Waves,
};

function CliIcon({ name, className }: { name: string; className?: string }) {
  const Icon = iconMap[name];
  if (!Icon) return <CircleDot className={className} />;
  return <Icon className={className} />;
}

interface CliCardProps {
  cli: CliInfo;
  status: CliStatusResult | undefined;
  loading: boolean;
  syncing: boolean;
  restoring: boolean;
  installing: boolean;
  model: string;
  onModelChange: (model: string) => void;
  apiModels: string[];
  modelsLoading: boolean;
  onSync: () => void;
  onRestore: () => void;
  onViewConfig: () => void;
  onInstall: () => void;
  onDownload: () => void;
}

export function CliCard({
  cli,
  status,
  loading,
  syncing,
  restoring,
  installing,
  model,
  onModelChange,
  apiModels,
  modelsLoading,
  onSync,
  onRestore,
  onViewConfig,
  onInstall,
  onDownload,
}: CliCardProps) {
  const { t } = useTranslation();

  const installed = status?.installed ?? false;
  const version = status?.version;
  const isSynced = status?.is_synced ?? false;
  const hasBackup = status?.has_backup ?? false;
  const syncedCount = status?.synced_count;

  const canAutoInstall = cli.installType === "npm" || cli.installType === "vscode";

  return (
    <div
      className={`card card-compact bg-base-100 shadow-sm border border-base-300 transition-all hover:shadow-md ${
        !installed ? "opacity-60" : ""
      }`}
    >
      <div className="card-body gap-1.5 p-3">
        {/* Header row: icon + name + status */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <div className={`w-7 h-7 rounded-lg flex items-center justify-center ${
              isSynced ? "bg-success/10 text-success" : installed ? "bg-primary/10 text-primary" : "bg-base-300 text-base-content/30"
            }`}>
              <CliIcon name={cli.icon} className="w-3.5 h-3.5" />
            </div>
            <div>
              <span className="font-semibold text-sm leading-tight">{cli.name}</span>
              {version && (
                <span className="text-[10px] opacity-40 ml-1.5">v{version}</span>
              )}
            </div>
          </div>

          {/* Status badge */}
          <div>
            {loading ? (
              <span className="loading loading-dots loading-xs opacity-40" />
            ) : !installed ? (
              <span className="badge badge-ghost badge-xs">{t("cli.notDetected")}</span>
            ) : isSynced ? (
              <span className="badge badge-success badge-xs gap-0.5">
                <Check className="w-2.5 h-2.5" />
                {t("cli.synced")}
              </span>
            ) : (
              <span className="badge badge-warning badge-xs">{t("cli.notSynced")}</span>
            )}
          </div>
        </div>

        {/* Not installed: install/download action */}
        {!installed && !loading && (
          <div className="flex items-center gap-2 mt-0.5">
            {canAutoInstall ? (
              <button
                className="btn btn-primary btn-xs flex-1"
                onClick={onInstall}
                disabled={installing}
              >
                {installing && <span className="loading loading-spinner loading-xs" />}
                {installing ? t("install.installing") : t("install.install")}
              </button>
            ) : (
              <button
                className="btn btn-outline btn-xs flex-1"
                onClick={onDownload}
              >
                {t("install.download")}
              </button>
            )}
          </div>
        )}

        {/* Installed: sync controls */}
        {installed && (
          <>
            {/* Model selector for syncable tools */}
            {cli.installType !== "manual-config" && (
              <div className="flex items-center gap-2 mt-0.5">
                <ModelSelector
                  value={model}
                  onChange={onModelChange}
                  apiModels={apiModels}
                  modelsLoading={modelsLoading}
                  size="xs"
                />
              </div>
            )}

            {/* Synced models count */}
            {syncedCount != null && syncedCount > 0 && (
              <div className="text-[10px] opacity-40">
                {t("cli.syncedModels", { count: syncedCount })}
              </div>
            )}

            {/* Action buttons */}
            {cli.installType !== "manual-config" ? (
              <div className="flex gap-1 mt-0.5">
                <button
                  className={`btn btn-xs flex-1 ${isSynced ? "btn-outline btn-success" : "btn-primary"}`}
                  onClick={onSync}
                  disabled={syncing}
                >
                  {syncing && <span className="loading loading-spinner loading-xs" />}
                  {t("cli.sync")}
                </button>
                <button
                  className="btn btn-ghost btn-xs"
                  onClick={onRestore}
                  disabled={restoring || !hasBackup}
                  title={t("cli.restore")}
                >
                  {restoring && <span className="loading loading-spinner loading-xs" />}
                  {t("cli.restore")}
                </button>
                <button
                  className="btn btn-ghost btn-xs"
                  onClick={onViewConfig}
                  title={t("cli.viewConfig")}
                >
                  {t("cli.viewConfig")}
                </button>
              </div>
            ) : (
              <div className="text-[10px] opacity-40 mt-0.5">
                {t("install.manualConfigHint")}
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}
