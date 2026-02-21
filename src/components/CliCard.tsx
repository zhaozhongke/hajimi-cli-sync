import { useTranslation } from "react-i18next";
import {
  Terminal, Code, Sparkles, FileCode, Bot, MousePointer,
  MessageSquare, Cherry, Cpu, FileText, Rabbit, Ruler,
  Beer, Brain, Zap, Waves, Check, CircleDot, Info, ExternalLink,
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
  isSwitching?: boolean;
  model: string;
  onModelChange: (model: string) => void;
  apiModels: string[];
  modelsLoading: boolean;
  onSync: () => void;
  onRestore: () => void;
  onViewConfig: () => void;
  onOpenDownload?: () => void;
  onLaunch?: () => void;
  onCommunity?: () => void;
}

export function CliCard({
  cli,
  status,
  loading,
  syncing,
  restoring,
  isSwitching = false,
  model,
  onModelChange,
  apiModels,
  modelsLoading,
  onSync,
  onRestore,
  onViewConfig,
  onOpenDownload,
  onLaunch,
  onCommunity,
}: CliCardProps) {
  const { t } = useTranslation();

  const installed = status?.installed ?? false;
  const version = status?.version;
  const isSynced = status?.is_synced ?? false;
  const hasBackup = status?.has_backup ?? false;
  const syncedCount = status?.synced_count;

  const busy = syncing || restoring || isSwitching;

  return (
    <div
      className={`card card-compact glass-card card-hover shadow-sm transition-all ${
        !installed ? "opacity-50" : ""
      }`}
    >
      <div className="card-body gap-2 p-3.5">
        {/* Header row: icon + name + status */}
        <div className="flex items-center justify-between gap-2">
          <div className="flex items-center gap-2.5 min-w-0">
            <div className={`w-8 h-8 rounded-xl flex items-center justify-center shrink-0 transition-colors ${
              isSynced
                ? "bg-success/15 text-success"
                : installed
                ? "bg-primary/10 text-primary"
                : "bg-base-300/50 text-base-content/25"
            }`}>
              <CliIcon name={cli.icon} className="w-4 h-4" />
            </div>
            <div className="min-w-0">
              <div className="flex items-baseline gap-1.5">
                <span className="font-semibold text-sm leading-tight truncate">{cli.name}</span>
                {version && (
                  <span className="text-[10px] opacity-35 font-mono">{t("cli.version", { version })}</span>
                )}
                {onCommunity && (
                  <button
                    className="opacity-30 hover:opacity-70 transition-opacity"
                    onClick={(e) => { e.stopPropagation(); onCommunity(); }}
                    title={t("cli.viewCommunity")}
                  >
                    <ExternalLink className="w-3 h-3" />
                  </button>
                )}
              </div>
              {cli.descKey && (
                <p className="text-[10px] opacity-45 leading-tight mt-0.5 line-clamp-2">{t(cli.descKey)}</p>
              )}
            </div>
          </div>

          {/* Status badge */}
          <div className="shrink-0">
            {loading ? (
              <span className="loading loading-dots loading-xs opacity-40" />
            ) : !installed ? (
              <span className="badge badge-ghost badge-xs whitespace-nowrap">{t("cli.notDetected")}</span>
            ) : isSynced ? (
              <span className="badge badge-success badge-xs gap-0.5 whitespace-nowrap">
                <Check className="w-2.5 h-2.5" />
                {t("cli.synced")}
              </span>
            ) : (
              <span className="badge badge-warning badge-xs whitespace-nowrap">{t("cli.notSynced")}</span>
            )}
          </div>
        </div>

        {/* Not installed: hint + link to official site */}
        {!installed && !loading && (
          <div className="flex items-center gap-2 mt-0.5">
            <span className="text-[10px] opacity-40 flex-1">{t("cli.notDetectedHint")}</span>
            {onOpenDownload && (
              <button
                className="btn btn-ghost btn-xs opacity-50 hover:opacity-100 transition-opacity shrink-0"
                onClick={onOpenDownload}
                title={t("cli.goToSite")}
              >
                <ExternalLink className="w-3 h-3" />
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
              <div className="text-[10px] opacity-35 font-medium">
                {t("cli.syncedModels", { count: syncedCount })}
              </div>
            )}

            {/* Action buttons */}
            {cli.installType !== "manual-config" ? (
              <div className="flex gap-1.5 mt-0.5">
                <button
                  className={`btn btn-xs flex-1 shadow-sm ${isSynced ? "btn-outline btn-success" : "btn-primary"}`}
                  onClick={onSync}
                  disabled={busy}
                >
                  {syncing && <span className="loading loading-spinner loading-xs" />}
                  {t("cli.sync")}
                </button>
                <button
                  className="btn btn-ghost btn-xs opacity-70 hover:opacity-100"
                  onClick={onRestore}
                  disabled={busy || !hasBackup}
                  title={t("cli.restore")}
                >
                  {restoring && <span className="loading loading-spinner loading-xs" />}
                  {t("cli.restore")}
                </button>
                <button
                  className="btn btn-ghost btn-xs opacity-70 hover:opacity-100"
                  onClick={onViewConfig}
                  disabled={busy}
                  title={t("cli.viewConfig")}
                >
                  {t("cli.viewConfig")}
                </button>
                {onLaunch && cli.launchName && (
                  <button
                    className="btn btn-ghost btn-xs opacity-70 hover:opacity-100"
                    onClick={onLaunch}
                    title={t("cli.openApp")}
                  >
                    <ExternalLink className="w-3 h-3" />
                  </button>
                )}
              </div>
            ) : (
              <div className="flex items-center gap-1 mt-0.5">
                <span className="text-[10px] opacity-35 flex-1">{t("install.manualConfigHint")}</span>
                {onLaunch && cli.launchName && (
                  <button
                    className="btn btn-ghost btn-xs opacity-70 hover:opacity-100"
                    onClick={onLaunch}
                    title={t("cli.openApp")}
                  >
                    {t("cli.openApp")}
                    <ExternalLink className="w-3 h-3" />
                  </button>
                )}
              </div>
            )}

            {/* Post-sync hint: tell user what they still need to do */}
            {cli.postSyncHintKey && isSynced && cli.installType !== "manual-config" && (
              <div className="flex items-start gap-1.5 mt-1.5 p-2 rounded-lg bg-info/8 text-info border border-info/10">
                <Info className="w-3 h-3 mt-0.5 shrink-0" />
                <span className="text-[10px] leading-relaxed">{t(cli.postSyncHintKey)}</span>
              </div>
            )}

            {/* Manual-config hint always visible */}
            {cli.postSyncHintKey && cli.installType === "manual-config" && (
              <div className="flex items-start gap-1.5 mt-1.5 p-2 rounded-lg bg-warning/8 text-warning border border-warning/10">
                <Info className="w-3 h-3 mt-0.5 shrink-0" />
                <span className="text-[10px] leading-relaxed">{t(cli.postSyncHintKey)}</span>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}
