/** The input is invalid (e.g. a malformed identifier or out of range coordinates) */
export class ValidationError extends Error {
  override readonly name = "ValidationError";
}

/** The requested item does not exist in the navigation data */
export class NotFoundError extends Error {
  override readonly name = "NotFoundError";
}
