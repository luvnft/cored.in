import { ROUTES } from "@/router/routes";
import { Box, Button, Flex, IconButton } from "@chakra-ui/react";
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
import { useCustomToast } from "@/hooks";

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
  const { successToast } = useCustomToast();

  const postUrl = ROUTES.USER.POST.buildPath(post.creatorWallet, post.id);

  const handleShare = () => {
    navigator.clipboard.writeText(window.location.origin + postUrl);
    successToast("Post URL copied to clipboard");
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
        color={isLiked ? "brand.400" : "text.700"}
        _hover={{ color: "brand.400" }}
        leftIcon={
          isLiked ? (
            <FaHeart fontSize="1.25rem" />
          ) : (
            <FaRegHeart fontSize="1.25rem" />
          )
        }
        onClick={handleLike}
        // isLoading={isLiking}
      >
        <Box as="span" opacity={isLiking ? 0 : 1} transition={"opacity 2s"}>
          {post.likes}
        </Box>
      </Button>
      <IconButton
        icon={<FaRegComment fontSize="1.25rem" />}
        variant="empty"
        aria-label="Add comment."
        fontSize="1rem"
        color={opened ? "brand.300" : "text.700"}
        onClick={handleComment}
        isLoading={isDetailLoading}
      />

      <IconButton
        icon={<FaRetweet fontSize="1.5rem" />}
        variant="empty"
        aria-label="Share."
        size="1rem"
        color="text.700"
        onClick={handleShare}
      />

      <IconButton
        as={ReactRouterLink}
        icon={<FaRegEye fontSize="1.25rem" />}
        to={postUrl}
        variant="empty"
        aria-label="Add comment."
        fontSize="1rem"
        color="text.700"
      />
    </Flex>
  );
};
