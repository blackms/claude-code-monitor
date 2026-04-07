import { NextResponse } from "next/server";
import { loadDashboardPayload } from "@/lib/load-claude-data";

export const dynamic = "force-dynamic";

export async function GET() {
  const data = await loadDashboardPayload();
  return NextResponse.json(data);
}
