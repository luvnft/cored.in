import { Controller, Get, Param, Req, UseGuards } from "@nestjs/common";
import { PostsService } from "./posts.service";
import { AuthenticatedRequest } from "../authentication";
import { LoggedIn } from "../authentication/guard";
import { TypedBody, TypedRoute } from "@nestia/core";
import { CreatePostDTO } from "@coredin/shared";

@Controller("posts")
@UseGuards(LoggedIn)
export class PostsController {
  constructor(private readonly postsService: PostsService) {}

  @Get()
  async getPublic() {
    return this.postsService.getPublic();
  }

  @Get(":owner")
  async getOwn(
    @Param("owner") owner: string,
    @Req() req: AuthenticatedRequest
  ) {
    // TODO - check for valid profile subscription
    if (owner === req.wallet) {
      return this.postsService.getAllUserPosts(req.wallet);
    }

    return this.postsService.getPublicUserPosts(owner);
  }

  @TypedRoute.Post()
  async updateProfile(
    @Req() req: AuthenticatedRequest,
    @TypedBody() createPostDTO: CreatePostDTO
  ) {
    // TODO - handle relevant return type and potential errors
    return await this.postsService.create(req.wallet, createPostDTO);
  }
}
