import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import "./App.css";

const appWindow = getCurrentWebviewWindow();

interface SystemItem {
  name: string;
  kind: string;
  path: string;
}

function App() {
  const [input, setInput] = useState("");
  const [aiResponse, setAiResponse] = useState("");
  const [systemResults, setSystemResults] = useState<SystemItem[]>([]);
  const [activeTab, setActiveTab] = useState("ai");
  const inputRef = useRef<HTMLInputElement>(null);

  const handleSearch = async (query: string) => {
    if (!query.trim()) {
      setAiResponse("");
      setSystemResults([]);
      return;
    }

    if (activeTab === "ai") {
      const res: string = await invoke("ask_newtron", { message: query });
      setAiResponse(res);
      setSystemResults([]);
    } else {
      const results: SystemItem[] = await invoke("get_system_results", { query });
      setSystemResults(results);
      setAiResponse("");
    }
  };

  useEffect(() => {
    inputRef.current?.focus();
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        appWindow.hide();
      }
      if (e.key === "Enter" && !e.shiftKey) {
        handleSearch(input);
      }
    };
    window.addEventListener("keydown", handleKeyDown, true);
    return () => window.removeEventListener("keydown", handleKeyDown, true);
  }, [input, activeTab]);

  return (
    <div className="wrapper">
      <div className="main-container">
        <div className="search-box">
          <span className="ai-icon">✨</span>
          <input
            ref={inputRef}
            type="text"
            value={input}
            onChange={(e) => {
              setInput(e.target.value);
              handleSearch(e.target.value);
            }}
            placeholder="Search or ask AI..."
            className="search-input"
          />
        </div>

        <div className="options-container">
          <button 
            className={activeTab === "files" ? "active" : ""} 
            onClick={() => setActiveTab("files")}
          >
            1 Search Files
          </button>
          <button 
            className={activeTab === "ai" ? "active" : ""} 
            onClick={() => setActiveTab("ai")}
          >
            2 Ask AI
          </button>
          <button 
            className={activeTab === "web" ? "active" : ""} 
            onClick={() => setActiveTab("web")}
          >
            3 Search Web
          </button>
        </div>

        <div className={`results-wrapper ${(aiResponse || systemResults.length > 0) ? "expanded" : ""}`}>
          <div className="results-inner">
            {aiResponse && (
              <div className="ai-result-block">
                <div className="section-title">Newtron AI</div>
                <div className="ai-content">{aiResponse}</div>
              </div>
            )}
            
            {systemResults.length > 0 && (
              <div className="system-results-block">
                <div className="section-title">System Results</div>
                {systemResults.map((item, idx) => (
                  <div key={idx} className="system-row">
                    <div className="item-icon-box">{item.kind[0]}</div>
                    <div className="item-details">
                      <div className="item-name">{item.name}</div>
                      <div className="item-path">{item.path}</div>
                    </div>
                    <div className="item-tag">{item.kind}</div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;