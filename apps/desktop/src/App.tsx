import { BrowserRouter, Route, Routes } from "react-router-dom";
import AppShell from "./components/AppShell";
import { ToastProvider } from "./components/Toast";
import HelpPage from "./pages/HelpPage";
import TimelinePage from "./pages/TimelinePage";
import WorkspaceListPage from "./pages/WorkspaceListPage";

export default function App() {
  return (
    <ToastProvider>
      <BrowserRouter>
        <Routes>
          <Route element={<AppShell />}>
            <Route path="/" element={<WorkspaceListPage />} />
            <Route path="/help" element={<HelpPage />} />
            <Route path="/workspace/:workspaceId" element={<TimelinePage />} />
          </Route>
        </Routes>
      </BrowserRouter>
    </ToastProvider>
  );
}
