export interface Dioxus {
  send(msg: unknown): void;
  recv(): Promise<unknown>;
}
