import { HistoryPanel } from "./components/HistoryPanel";
import { TranscriptView } from "./components/TranscriptView";
import { MetaPanel } from "./components/MetaPanel";

export function App() {
  return (
    <div className="app-layout">
      <aside className="panel-left">
        <HistoryPanel />
      </aside>
      <main className="panel-center">
        <TranscriptView />
      </main>
      <aside className="panel-right">
        <MetaPanel />
      </aside>
    </div>
  );
}
