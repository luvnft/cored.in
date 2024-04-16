import { Login } from "@/components";
import { Box, Flex, Heading, Text } from "@chakra-ui/react";

const AppPage = () => {
  return (
    <Box mx="auto" maxW="1680px">
      <Flex
        direction="column"
        justify="center"
        align="center"
        minH="100vh"
        px="2em"
        gap="5em"
        mb="16"
      >
        <Heading as="h1" fontSize="4rem">
          Cored.
          <Text display="inline" color="brand.600">
            in
          </Text>
        </Heading>
        <Login />
      </Flex>
    </Box>
  );
};

export default AppPage;
