import { DashboardApp } from "@/components/dashboard-app";
import { loadDashboardPayload } from "@/lib/load-claude-data";

export const dynamic = "force-dynamic";

export default async function Home() {
  const initial = await loadDashboardPayload();
  return <DashboardApp initial={initial} />;
}
