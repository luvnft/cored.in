import { extendTheme } from "@chakra-ui/react";
import "@fontsource/space-grotesk";
import "@fontsource/noto-sans";
import {
  buttonTheme,
  inputTheme,
  layerStyles,
  menuTheme,
  stepperTheme,
  tabsTheme,
  textStyles
} from "./themes";

const fonts = {
  heading: `'Space Grotesk', sans-serif`,
  body: `"Noto Sans", sans-serif`
};

const styles = {
  global: {
    "::-webkit-scrollbar": {
      width: "12px",
      backgroundColor: "background.800"
    },
    "::-webkit-scrollbar-thumb": {
      backgroundColor: "background.600",
      borderRadius: "6px"
    },
    "::-webkit-scrollbar-thumb:hover": {
      backgroundColor: "background.100"
    },
    "html, body": {
      color: "text.100",
      boxSizing: "border-box"
    },
    body: {
      background: "background.900"
    },
    html: {
      scrollBehavior: "smooth"
    },
    a: {
      textUnderlineOffset: "0.5em"
    },
    p: textStyles.md
  }
};

const colors = {
  brand: {
    500: "#7AF9B3" // green / main
  },
  complimentary: {
    500: "#1F45F3" // blue but not readable // do not use yet
  },
  text: {
    100: "#F1F1F0", // light - white / main text
    300: "#b0b0b0", // hover / text in the stepper
    400: "#828178", // medium / text on card
    600: "#737373", // - hover 2 / not in use yet
    700: "#75746B", // heading in profile
    800: "#62625D", // - medium - dark / not in use yet
    900: "#121111" // dark
  },
  background: {
    100: "#ebebeb", // - not in use yet
    400: "#828178", /// === text.400
    600: "#3E3D3A", // card bg
    700: "#272723",
    800: "#222320", // very dark bg / profile bg
    900: "#1C1C1A" // body bg / black
  }
};

const components = {
  Button: buttonTheme,
  Stepper: stepperTheme,
  Input: inputTheme,
  Menu: menuTheme,
  Tabs: tabsTheme
};

const breakpoints = {
  base: "0em", // 0px
  sm: "30em", // ~480px. em is a relative unit and is dependant on the font size.
  md: "48em", // ~768px
  lg: "62em", // ~992px
  xl: "80em", // ~1280px
  "2xl": "96em" // ~1536px
};

export const theme = extendTheme({
  fonts,
  styles,
  colors,
  components,
  breakpoints,
  layerStyles,
  textStyles
});
