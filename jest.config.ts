import type { JestConfigWithTsJest } from "ts-jest/dist/types"

module.exports = <JestConfigWithTsJest>{
  testEnvironment: "node",
  transform: {
    "^.+\\.tsx?$": "ts-jest",
  },
  testRegex: "(/__tests__/.*|(\\.|/)(test|spec))\\.(jsx?|tsx?)$",
  moduleFileExtensions: ["ts", "js", "json", "node"],
  setupFilesAfterEnv: ["./src/test/setup.ts"],
}
