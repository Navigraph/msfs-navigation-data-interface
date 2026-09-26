import { expect } from "bun:test";
import type { Server } from "node:http";
import type { AddressInfo } from "node:net";
import { createApp } from "../src/app.module";
import type { NavigationDataSource } from "../src/NavigationDataService";

/** Asserts that the promise rejects with an instance of the given error class */
export async function expectRejection(promise: Promise<unknown>, errorClass: new (...args: never[]) => Error) {
  const error = await promise.then(
    () => null,
    (e: unknown) => e,
  );
  expect(error).toBeInstanceOf(errorClass);
}

/** Starts the app with the given navigation data on a free port. Close it with `app.close()`. */
export async function startApp(source: NavigationDataSource) {
  const app = await createApp(source, { logger: false });
  await app.listen(0);
  const { port } = (app.getHttpServer() as Server).address() as AddressInfo;
  return { app, baseUrl: `http://localhost:${port}` };
}

/** Reads a JSON response body as the given type */
export async function readJson<T>(res: Response): Promise<T> {
  return (await res.json()) as T;
}

/** The error body of Nest */
export interface ErrorResponse {
  statusCode: number;
  message: string;
  error?: string;
}
