// src/invoke.ts
import { commands, type Result } from "@/types/bindings";

// All generated command functions (from tauri-specta)
type Commands = typeof commands;
type CmdName = keyof Commands;
type AwaitedReturn<F> = F extends (...a: any[]) => Promise<infer R> ? R : never;

// Extract arg list and ok-payload types from the generated functions
type ArgsOf<K extends CmdName> = Parameters<Commands[K]>;
type OkOf<K extends CmdName> =
  AwaitedReturn<Commands[K]> extends Result<infer T, any>
    ? T
    : AwaitedReturn<Commands[K]>;

export class TauriCommandError extends Error {
  constructor(
    public readonly command: string,
    public readonly detail?: unknown,
  ) {
    super(typeof detail === "string" ? detail : `Command "${command}" failed`);
    this.name = "TauriCommandError";
  }
}

/**
 * invoke:
 * - Command name is type-checked against your generated bindings
 * - Args are type-checked against the generated signature
 * - Unwraps `Result<T, E>` into `T`, throwing `TauriCommandError` on error
 */
export async function invoke<K extends CmdName>(
  command: K,
  ...args: ArgsOf<K>
): Promise<OkOf<K>> {
  // Call the generated function (which already uses TAURI_INVOKE under the hood)
  // Works for both 0-arg and 1-arg commands, since ArgsOf<K> reflects that.
  const raw = (await (commands[command] as any)(...args)) as AwaitedReturn<
    Commands[K]
  >;

  // Generated commands return Specta Result<T, E>
  if (raw && typeof raw === "object" && "status" in (raw as any)) {
    const r = raw as Result<unknown, unknown>;
    if (r.status === "error") {
      throw new TauriCommandError(String(command), r.error);
    }
    return r.data as OkOf<K>;
  }

  // (If you ever return plain T from Rust, pass-through still works.)
  return raw as OkOf<K>;
}
