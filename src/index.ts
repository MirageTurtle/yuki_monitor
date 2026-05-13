import { MirrorsStatusChecker } from "./mirrors";
import { TelegramClient } from "./telegram";

export interface Env {
  TELEGRAM_BOT_TOKEN: string;
  TELEGRAM_CHAT_ID: string;
  REPO_WHITELIST?: string;
}

const THRESHOLD_DAYS = 7;

function parseWhitelist(raw: string | undefined): Set<string> {
  if (!raw) return new Set();
  return new Set(
    raw
      .split(",")
      .map((s) => s.trim())
      .filter((s) => s.length > 0)
  );
}

async function runCheck(env: Env): Promise<string> {
  const whitelist = parseWhitelist(env.REPO_WHITELIST);
  const checker = new MirrorsStatusChecker(THRESHOLD_DAYS, whitelist);
  const outdated = await checker.check();

  if (outdated.length === 0) {
    return "OK: all mirrors are up to date";
  }

  const names = outdated.map((e) => e.name).join(", ");
  const message = `*[USTC LUG Mirrors]* Repo(s) failed to sync more than ${THRESHOLD_DAYS} days: ${names}`;

  const telegram = new TelegramClient(env.TELEGRAM_BOT_TOKEN, env.TELEGRAM_CHAT_ID);
  await telegram.sendMessage(message);

  return `Alert sent for ${outdated.length} outdated repo(s): ${names}`;
}

export default {
  async scheduled(
    _event: ScheduledEvent,
    env: Env,
    _ctx: ExecutionContext
  ): Promise<void> {
    await runCheck(env);
  },

  async fetch(request: Request, env: Env): Promise<Response> {
    const result = await runCheck(env);
    return new Response(result, { status: 200 });
  },
};
