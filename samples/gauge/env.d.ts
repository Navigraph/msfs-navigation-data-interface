// Add secret to global env

declare namespace NodeJS {
  interface ProcessEnv {
    NG_CLIENT_ID: string
    NG_CLIENT_SECRET: string
  }
}
