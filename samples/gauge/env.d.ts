// Add secret to global env

declare namespace NodeJS {
  interface ProcessEnv {
    CLIENT_ID: string
    CLIENT_SECRET: string
  }
}
