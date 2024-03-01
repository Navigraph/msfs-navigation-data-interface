// Add secret to global env

declare namespace NodeJS {
  interface ProcessEnv {
    // For test running
    // Must be a client which supports password auth grants
    NAVIGATION_DATA_SIGNED_URL: string
  }
}
