import { Button, Flex, Heading, Link } from "@chakra-ui/react";
import { Link as ReactRouterLink } from "react-router-dom";
import { ROUTES } from "@/router/routes";

export const CallToAction = () => {
  return (
    <Flex flexDirection="column" justify="center" align="center">
      <Heading
        as="h2"
        fontSize={{ base: "3rem", md: "4rem" }}
        mb="2rem"
        textAlign="center"
      >
        Elevate your career
      </Heading>
      <Link as={ReactRouterLink} to={ROUTES.HOME.path}>
        <Button variant="primary" size="xl">
          Sign In
        </Button>
      </Link>
    </Flex>
  );
};
