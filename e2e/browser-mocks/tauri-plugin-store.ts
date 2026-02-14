/**
 * Mock for @tauri-apps/plugin-store
 */

const stores = new Map<string, Map<string, unknown>>();

function getOrCreateStore(name: string): Map<string, unknown> {
  if (!stores.has(name)) {
    stores.set(name, new Map());
  }
  return stores.get(name)!;
}

export async function load(name: string): Promise<{
  get: (key: string) => Promise<unknown>;
  set: (key: string, value: unknown) => Promise<void>;
  save: () => Promise<void>;
  delete: (key: string) => Promise<boolean>;
  clear: () => Promise<void>;
  keys: () => Promise<string[]>;
  values: () => Promise<unknown[]>;
  entries: () => Promise<[string, unknown][]>;
  length: () => Promise<number>;
  has: (key: string) => Promise<boolean>;
  onKeyChange: (key: string, cb: (value: unknown) => void) => Promise<() => void>;
  onChange: (cb: (key: string, value: unknown) => void) => Promise<() => void>;
}> {
  const store = getOrCreateStore(name);

  return {
    get: async (key: string) => store.get(key) ?? null,
    set: async (key: string, value: unknown) => { store.set(key, value); },
    save: async () => {},
    delete: async (key: string) => store.delete(key),
    clear: async () => { store.clear(); },
    keys: async () => Array.from(store.keys()),
    values: async () => Array.from(store.values()),
    entries: async () => Array.from(store.entries()),
    length: async () => store.size,
    has: async (key: string) => store.has(key),
    onKeyChange: async (_key: string, _cb: (value: unknown) => void) => () => {},
    onChange: async (_cb: (key: string, value: unknown) => void) => () => {},
  };
}
