/** @type {import('tailwindcss').Config} */
export default {
  content: ["./MyInstrument.html", "./MyInstrument.tsx", "./Components/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      fontFamily: {
        inter: ["Inter", "sans-serif"],
      },
      colors: {
        "ng-background": {
          900: "#0E131B",
          800: "#111721",
          700: "#151D29",
          600: "#192230",
          500: "#1D2838",
          400: "#222F42",
          300: "#27364D",
          200: "#2E3F59",
          100: "#324562",
          50: "#374C6C",
          25: "#3D5476",
        },
        blue: {
          900: "#113355",
          800: "#133B62",
          700: "#16426F",
          600: "#184A7B",
          500: "#1B5288",
          400: "#1D5995",
          300: "#1F5F9E",
          200: "#2064A6",
          100: "#2369AF",
          50: "#2571BB",
          25: "#287BCC",
          5: "#3788D7",
        },
        teal: {
          900: "#115555",
          800: "#136262",
          700: "#166F6F",
          600: "#187B7B",
          500: "#1B8888",
          400: "#1D9595",
          300: "#1F9E9E",
          200: "#22AFAF",
          100: "#24B7B7",
        },
        purple: {
          900: "#1B0E21",
          800: "#25132E",
          700: "#30193B",
          600: "#3A1E47",
          500: "#442454",
          400: "#4F2961",
          300: "#552C68",
          200: "#5B2F6F",
          100: "#633479",
          50: "#6C3984",
          25: "#753D8F",
          5: "#7E429A",
        },
        "blue-gray": {
          900: "#27313F",
          800: "#2B3645",
          700: "#2F3B4B",
          600: "#333F52",
          500: "#374458",
          400: "#3B495E",
          300: "#3F4E64",
          200: "#43536B",
          100: "#475871",
          50: "#4D5F7A",
        },
        gray: {
          900: "#5A6068",
          800: "#666D75",
          700: "#727982",
          600: "#7F868F",
          500: "#8E949C",
          400: "#9AA0A7",
          300: "#A8ADB3",
          200: "#B6BABF",
          100: "#C3C7CB",
          50: "#D1D3D7",
          25: "#DEE0E2",
          5: "#ECEDEE",
        },
        sid: {
          900: "#BA3476",
          800: "#C34080",
          700: "#CB4C8B",
          600: "#D45895",
          500: "#DC649F",
          400: "#E075A9",
          300: "#E485B4",
          200: "#E796BE",
          100: "#E99EC3",
        },
        star: {
          900: "#6CA550",
          800: "#78B15A",
          700: "#82BA67",
          600: "#8DC273",
          500: "#98CA7F",
          400: "#A4D08D",
          300: "#AFD69C",
          200: "#BBDCAA",
          100: "#C1DFB1",
        },
        app: {
          900: "#EC7B2C",
          800: "#EE8842",
          700: "#F09354",
          600: "#F19F67",
          500: "#F3AC7A",
          400: "#F5B78D",
          300: "#F6C3A0",
          200: "#F8CEB2",
          100: "#F9D4BC",
        },
        red: {
          900: "#EC7B2C",
          800: "#EE8842",
          700: "#F09354",
          600: "#F19F67",
          500: "#F3AC7A",
          400: "#F5B78D",
          300: "#F6C3A0",
          200: "#F8CEB2",
          100: "#F9D4BC",
        },
        orange: {
          900: "#794006",
          800: "#8D4A07",
          700: "#A05408",
          600: "#B35E09",
          500: "#C7690A",
          400: "#D8720B",
          300: "#ED7D0C",
          200: "#F3871B",
          100: "#F4912F",
          50: "#F59C42",
        },
        yellow: {
          900: "#A48B0A",
          800: "#B79B0B",
          700: "#CAAB0C",
          600: "#DEBC0D",
          500: "#F1CC0E",
          400: "#F2D021",
          300: "#F3D435",
          200: "#F4D848",
          100: "#F5DC5B",
        },
        rwy: {
          900: "#1AADEC",
          800: "#22B1EF",
          700: "#32B9F3",
          600: "#41C1F8",
          500: "#51C9FD",
          400: "#65CFFD",
          300: "#79D5FD",
          200: "#8EDCFE",
          100: "#98DFFE",
        },
      },
    },
  },
  corePlugins: {
    backdropOpacity: false,
    backgroundOpacity: false,
    borderOpacity: false,
    divideOpacity: false,
    ringOpacity: false,
    textOpacity: false,
  },
  plugins: [],
};
