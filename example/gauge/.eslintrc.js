module.exports = {
  extends: "../../.eslintrc",
  parserOptions: {
    tsconfigRootDir: __dirname,
    project: ["./tsconfig.json"],
    ecmaVersion: 2022,
    sourceType: "module",
    ecmaFeatures: { jsx: true },
    jsxPragma: "FSComponent",
  },
};
