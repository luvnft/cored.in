import { Box, Flex } from "@chakra-ui/react";
import { Outlet, ScrollRestoration } from "react-router-dom";
import { Disclaimer } from "@/components/Disclaimer";
import { Header } from "@/components";
import LandingBg35 from "@/assets/landing-bg-35.png";

export const LoginRoot = () => {
  return (
    <Flex
      direction="column"
      justify="start"
      minH="100dvh"
      bgImage={LandingBg35}
      bgPosition="center"
      // border="2px solid red"
    >
      <Header />
      <Box as="main" mb="auto">
        <Outlet />
        <ScrollRestoration />
      </Box>
      <Disclaimer />
    </Flex>
  );
};
