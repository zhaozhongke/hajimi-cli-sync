import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";

interface ConfigViewerProps {
  name: string;
  files: string[];
  getContent: (fileName?: string) => Promise<string>;
  onClose: () => void;
}

export function ConfigViewer({
  name,
  files,
  getContent,
  onClose,
}: ConfigViewerProps) {
  const { t } = useTranslation();
  const [selectedFile, setSelectedFile] = useState(files[0] || "");
  const [content, setContent] = useState("");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    setLoading(true);
    getContent(selectedFile).then((c) => {
      setContent(c);
      setLoading(false);
    });
  }, [selectedFile, getContent]);

  return (
    <dialog className="modal modal-open">
      <div className="modal-box max-w-lg max-h-[80vh]">
        <h3 className="font-bold text-lg mb-2">
          {t("config.title", { name })}
        </h3>

        {files.length > 1 && (
          <div className="tabs tabs-boxed mb-2">
            {files.map((f) => (
              <button
                key={f}
                className={`tab tab-sm ${selectedFile === f ? "tab-active" : ""}`}
                onClick={() => setSelectedFile(f)}
              >
                {f}
              </button>
            ))}
          </div>
        )}

        <div className="bg-base-200 rounded-lg p-3 overflow-auto max-h-[50vh]">
          {loading ? (
            <div className="flex justify-center py-4">
              <span className="loading loading-spinner loading-sm" />
            </div>
          ) : content ? (
            <pre className="text-xs font-mono whitespace-pre-wrap break-all">
              {content}
            </pre>
          ) : (
            <p className="text-sm opacity-60">{t("config.noContent")}</p>
          )}
        </div>

        <div className="modal-action">
          <button className="btn btn-sm" onClick={onClose}>
            {t("config.close")}
          </button>
        </div>
      </div>
      <form method="dialog" className="modal-backdrop">
        <button onClick={onClose}>close</button>
      </form>
    </dialog>
  );
}
