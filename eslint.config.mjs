import globals from "globals";
import pluginJs from "@eslint/js";
import tseslint from "typescript-eslint";
import prettierlint from "eslint-plugin-prettier/recommended";

export default tseslint.config(
  { ignores: ["**/dist/**"] },
  { languageOptions: { globals: globals.browser } },
  {
    rules: {
      ...pluginJs.configs.recommended.rules,
      semi: "off",
      curly: ["warn", "multi-line"],
      quotes: ["warn", "double", { avoidEscape: true }],
      "no-console": ["warn", { allow: ["warn", "error", "time", "timeEnd", "info"] }],
      "max-depth": ["warn", 4],
      "no-spaced-func": "off",
      "no-shadow": "off",
      "no-unused-vars": [
        "warn",
        {
          vars: "all",
          varsIgnorePattern: "^_",
          args: "after-used",
          argsIgnorePattern: "^_",
          ignoreRestSiblings: true,
          destructuredArrayIgnorePattern: "^_",
        },
      ],
    },
  },
  {
    files: ["**/*.{ts,tsx}"],
    extends: [tseslint.configs.recommendedTypeChecked, tseslint.configs.stylisticTypeChecked],
    languageOptions: { parser: tseslint.parser, parserOptions: { projectService: true } },
    rules: {
      "@typescript-eslint/no-shadow": "warn",
      "@typescript-eslint/ban-ts-comment": ["error", { "ts-ignore": "allow-with-description" }],
      "@typescript-eslint/no-empty-function": ["error", { allow: ["arrowFunctions"] }],
      "@typescript-eslint/no-explicit-any": "error",
      "@typescript-eslint/no-unused-vars": [
        "warn",
        {
          vars: "all",
          varsIgnorePattern: "^_",
          args: "after-used",
          argsIgnorePattern: "^_",
          ignoreRestSiblings: true,
          destructuredArrayIgnorePattern: "^_",
        },
      ],
      "@typescript-eslint/no-misused-promises": [
        "warn",
        { checksConditionals: true, checksVoidReturn: false, checksSpreads: true },
      ],
    },
  },
  { extends: [prettierlint], rules: { "prettier/prettier": "warn" } },
);
