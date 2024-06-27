import { ROUTES } from "@/router/routes";
import { Box, Button, Flex, IconButton, useToast } from "@chakra-ui/react";
import { FC } from "react";
import {
  FaHeart,
  FaRegComment,
  FaRegEye,
  FaRegHeart,
  FaRetweet
} from "react-icons/fa6";
import { Link as ReactRouterLink } from "react-router-dom";
import { PostDTO } from "@coredin/shared";

type PostActionBarProps = {
  post: PostDTO;
  opened: boolean;
  isLiked: boolean;
  isDetailLoading: boolean;
  handleComment: () => void;
  handleLike: () => void;
  isLiking: boolean;
};

export const ActionBar: FC<PostActionBarProps> = ({
  post,
  opened,
  isLiked,
  isDetailLoading,
  handleComment,
  handleLike,
  isLiking
}) => {
  const toast = useToast();

  const postUrl = ROUTES.USER.POST.buildPath(post.creatorWallet, post.id);

  const handleShare = () => {
    navigator.clipboard.writeText(window.location.origin + postUrl);
    toast({
      title: `Copied to clipboard`,
      duration: 3000,
      isClosable: true,
      render: () => (
        <Box
          mx="auto"
          color="text.100"
          p="1em 1.5em"
          bg="background.600"
          textAlign="center"
          borderRadius="4px"
        >
          Post URL copied to clipboard
        </Box>
      )
    });
  };

  return (
    <Flex
      w="100%"
      ml="auto"
      justify="space-between"
      // border="1px solid red"
    >
      <Button
        variant="empty"
        aria-label="Like the post."
        size="1rem"
        color={isLiked ? "brand.500" : "text.400"}
        leftIcon={
          isLiked ? (
            <FaHeart fontSize="1.25rem" />
          ) : (
            <FaRegHeart fontSize="1.25rem" />
          )
        }
        onClick={handleLike}
        isLoading={isLiking}
      >
        {post.likes}
      </Button>
      <IconButton
        icon={<FaRegComment fontSize="1.25rem" />}
        variant="empty"
        aria-label="Add comment."
        fontSize="1rem"
        color={opened ? "text.100" : "text.400"}
        onClick={handleComment}
        isLoading={isDetailLoading}
      />

      <IconButton
        icon={<FaRetweet fontSize="1.5rem" />}
        variant="empty"
        aria-label="Share."
        size="1rem"
        color="text.400"
        onClick={handleShare}
      />

      <IconButton
        as={ReactRouterLink}
        icon={<FaRegEye fontSize="1.25rem" />}
        to={postUrl}
        variant="empty"
        aria-label="Add comment."
        fontSize="1rem"
        color={"text.400"}
      />
    </Flex>
  );
};
