import { BrowserRouter, Route, Routes } from "react-router-dom";
import TimelinePage from "./pages/TimelinePage";
import WorkspaceListPage from "./pages/WorkspaceListPage";

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<WorkspaceListPage />} />
        <Route path="/workspace/:workspaceId" element={<TimelinePage />} />
      </Routes>
    </BrowserRouter>
  );
}
