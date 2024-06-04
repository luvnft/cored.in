import { VStack } from "@chakra-ui/layout";
import { Post } from ".";
import { PostDTO } from "@coredin/shared";
import { FC } from "react";

type FeedProps = {
  posts: PostDTO[];
};

export const Feed: FC<FeedProps> = ({ posts }) => {
  return (
    <VStack spacing="1.5em" w="100%" mb="4em">
      {posts.map((post, i) => (
        <Post key={`post-${i}`} post={post} />
      ))}
    </VStack>
  );
};
