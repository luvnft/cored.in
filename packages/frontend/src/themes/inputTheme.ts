import { inputAnatomy } from "@chakra-ui/anatomy";
import { createMultiStyleConfigHelpers } from "@chakra-ui/react";

const { definePartsStyle, defineMultiStyleConfig } =
  createMultiStyleConfigHelpers(inputAnatomy.keys);

const baseStyle = definePartsStyle({
  field: {
    color: "brand.900",
    fontFamily: "body",
    fontSize: { base: "0.875rem", lg: "1rem" },
    textAlign: "start",
    _placeholder: {
      color: "text.700"
    }
  }
});

export const inputTheme = defineMultiStyleConfig({ baseStyle });
