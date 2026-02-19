import { useState, useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { save, confirm } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import { invoke } from "@tauri-apps/api/core";
import { Copy, Download, Pencil, Save, X, FolderOpen, Check } from "lucide-react";
import hljs from "highlight.js/lib/core";
import json from "highlight.js/lib/languages/json";
import ini from "highlight.js/lib/languages/ini";
import "highlight.js/styles/github-dark.css";

// Register languages
hljs.registerLanguage("json", json);
hljs.registerLanguage("ini", ini);
hljs.registerLanguage("env", ini);
hljs.registerLanguage("toml", ini);

interface ConfigViewerProps {
  name: string;
  files: string[];
  getContent: (fileName?: string) => Promise<string>;
  onClose: () => void;
  cliId: string; // Add CLI ID for writing config
}

export function ConfigViewer({
  name,
  files,
  getContent,
  onClose,
  cliId,
}: ConfigViewerProps) {
  const { t } = useTranslation();
  const [selectedFile, setSelectedFile] = useState(files[0] || "");
  const [content, setContent] = useState("");
  const [editedContent, setEditedContent] = useState("");
  const [loading, setLoading] = useState(true);
  const [copying, setCopying] = useState(false);
  const [exporting, setExporting] = useState(false);
  const [saving, setSaving] = useState(false);
  const [editing, setEditing] = useState(false);
  const [validationError, setValidationError] = useState("");
  const codeRef = useRef<HTMLElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const getLanguage = (fileName: string): string => {
    const ext = fileName.split(".").pop()?.toLowerCase();
    switch (ext) {
      case "json":
        return "json";
      case "toml":
        return "toml";
      case "env":
        return "env";
      default:
        return "plaintext";
    }
  };

  useEffect(() => {
    setLoading(true);
    setEditing(false);
    setValidationError("");
    getContent(selectedFile).then((c) => {
      setContent(c);
      setEditedContent(c);
      setLoading(false);
    });
  }, [selectedFile, getContent]);

  useEffect(() => {
    if (codeRef.current && content && !editing) {
      const language = getLanguage(selectedFile);
      if (language !== "plaintext") {
        codeRef.current.removeAttribute("data-highlighted");
        hljs.highlightElement(codeRef.current);
      }
    }
  }, [content, selectedFile, editing]);

  const validateContent = (value: string): boolean => {
    const language = getLanguage(selectedFile);
    setValidationError("");

    if (language === "json") {
      try {
        JSON.parse(value);
        return true;
      } catch (err) {
        setValidationError(t("config.invalidJson") + ": " + (err as Error).message);
        return false;
      }
    }

    // TOML and ENV don't need strict validation here
    return true;
  };

  const handleEdit = () => {
    setEditing(true);
    setEditedContent(content);
    setTimeout(() => {
      if (textareaRef.current) {
        textareaRef.current.focus();
      }
    }, 100);
  };

  const handleCancel = () => {
    setEditing(false);
    setEditedContent(content);
    setValidationError("");
  };

  const handleSave = async () => {
    if (!validateContent(editedContent)) {
      return;
    }

    const confirmed = await confirm(
      t("config.saveConfirm", { file: selectedFile }),
      { title: t("config.saveTitle"), kind: "warning" }
    );

    if (!confirmed) {
      return;
    }

    setSaving(true);
    try {
      await invoke("write_config_file", {
        app: cliId,
        fileName: selectedFile,
        content: editedContent,
      });

      setContent(editedContent);
      setEditing(false);
      setValidationError("");
    } catch (err) {
      setValidationError(t("config.saveFailed") + ": " + err);
    } finally {
      setSaving(false);
    }
  };

  const handleCopy = async () => {
    const textToCopy = editing ? editedContent : content;
    if (!textToCopy) return;
    setCopying(true);
    try {
      await navigator.clipboard.writeText(textToCopy);
      setTimeout(() => setCopying(false), 2000);
    } catch (err) {
      console.error("Failed to copy:", err);
      setCopying(false);
    }
  };

  const handleExport = async () => {
    const textToExport = editing ? editedContent : content;
    if (!textToExport) return;
    setExporting(true);
    try {
      const filePath = await save({
        defaultPath: selectedFile,
        filters: [
          { name: "Config Files", extensions: ["json", "toml", "env", "txt"] },
          { name: "All Files", extensions: ["*"] },
        ],
      });
      if (filePath) {
        await writeTextFile(filePath, textToExport);
      }
    } catch (err) {
      console.error("Failed to export:", err);
    } finally {
      setExporting(false);
    }
  };

  return (
    <dialog className="modal modal-open">
      <div className="modal-box max-w-3xl max-h-[85vh]">
        <h3 className="font-bold text-lg mb-2">
          {t("config.title", { name })}
          {editing && (
            <span className="ml-2 badge badge-warning badge-sm">
              {t("config.editing")}
            </span>
          )}
        </h3>

        {files.length > 1 && (
          <div className="tabs tabs-boxed mb-2">
            {files.map((f) => (
              <button
                key={f}
                className={`tab tab-sm ${selectedFile === f ? "tab-active" : ""}`}
                onClick={() => setSelectedFile(f)}
                disabled={editing}
              >
                {f}
              </button>
            ))}
          </div>
        )}

        {validationError && (
          <div className="alert alert-error text-xs py-2 mb-2">
            <span>{validationError}</span>
          </div>
        )}

        <div className="bg-base-300 rounded-lg p-4 overflow-auto max-h-[60vh]">
          {loading ? (
            <div className="flex justify-center py-4">
              <span className="loading loading-spinner loading-sm" />
            </div>
          ) : editing ? (
            <textarea
              ref={textareaRef}
              className="textarea textarea-bordered w-full font-mono text-xs min-h-[50vh]"
              value={editedContent}
              onChange={(e) => setEditedContent(e.target.value)}
              spellCheck={false}
            />
          ) : content ? (
            <pre className="text-xs">
              <code
                ref={codeRef}
                className={`language-${getLanguage(selectedFile)}`}
              >
                {content}
              </code>
            </pre>
          ) : (
            <p className="text-sm opacity-60">{t("config.noContent")}</p>
          )}
        </div>

        <div className="modal-action">
          {editing ? (
            <>
              <button
                className="btn btn-sm btn-success gap-1"
                onClick={handleSave}
                disabled={saving || !!validationError}
              >
                {saving ? (
                  <span className="loading loading-spinner loading-xs" />
                ) : (
                  <Save className="w-3.5 h-3.5" />
                )}
                {t("config.save")}
              </button>
              <button
                className="btn btn-sm btn-ghost gap-1"
                onClick={handleCancel}
                disabled={saving}
              >
                <X className="w-3.5 h-3.5" />
                {t("config.cancel")}
              </button>
            </>
          ) : (
            <>
              <button
                className="btn btn-sm btn-primary gap-1"
                onClick={handleEdit}
                disabled={!content}
              >
                <Pencil className="w-3.5 h-3.5" />
                {t("config.edit")}
              </button>
              <button
                className="btn btn-sm btn-ghost gap-1"
                onClick={handleCopy}
                disabled={!content || copying}
              >
                {copying ? <Check className="w-3.5 h-3.5" /> : <Copy className="w-3.5 h-3.5" />}
                {copying ? t("config.copied") : t("config.copy")}
              </button>
              <button
                className="btn btn-sm btn-ghost gap-1"
                onClick={handleExport}
                disabled={!content || exporting}
              >
                {exporting ? (
                  <span className="loading loading-spinner loading-xs" />
                ) : (
                  <Download className="w-3.5 h-3.5" />
                )}
                {t("config.export")}
              </button>
              <button
                className="btn btn-sm btn-ghost gap-1"
                onClick={() => invoke("open_config_folder", { app: cliId })}
                title={t("config.openFolder")}
              >
                <FolderOpen className="w-3.5 h-3.5" />
                {t("config.openFolder")}
              </button>
              <button className="btn btn-sm" onClick={onClose}>
                {t("config.close")}
              </button>
            </>
          )}
        </div>
      </div>
      <form method="dialog" className="modal-backdrop">
        <button onClick={onClose}>close</button>
      </form>
    </dialog>
  );
}
