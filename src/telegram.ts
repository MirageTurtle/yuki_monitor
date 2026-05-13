const TELEGRAM_API_BASE = "https://api.telegram.org";

export class TelegramClient {
  private botToken: string;
  private chatId: string;

  constructor(botToken: string, chatId: string) {
    this.botToken = botToken;
    this.chatId = chatId;
  }

  async sendMessage(text: string): Promise<void> {
    const url = `${TELEGRAM_API_BASE}/bot${this.botToken}/sendMessage`;
    const body = JSON.stringify({
      chat_id: this.chatId,
      text,
      parse_mode: "Markdown",
    });

    const resp = await fetch(url, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body,
    });

    if (!resp.ok) {
      throw new Error(
        `Telegram API returned HTTP ${resp.status}: ${resp.statusText}`
      );
    }

    const data = (await resp.json()) as {
      ok: boolean;
      description?: string;
    };

    if (!data.ok) {
      throw new Error(
        `Telegram API error: ${data.description ?? "unknown error"}`
      );
    }
  }
}
