declare global {
  var Config: {
    get(key: string): string | null;
  };
}

export {};
