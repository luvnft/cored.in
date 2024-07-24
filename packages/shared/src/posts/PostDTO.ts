import { PostVisibility } from "./PostVisibility";

export type PostDTO = {
  id: number;
  creatorWallet: string;
  creatorUsername: string;
  creatorAvatar: string;
  creatorAvatarFallbackColor: string;
  visibility: PostVisibility;
  text: string;
  createdAt: string;
  likes: number;
  replyToPostId?: number;
};
