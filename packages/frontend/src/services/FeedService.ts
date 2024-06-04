import { HttpService } from "@/services";
import { CreatePostDTO, PostDTO, PostDetailDTO } from "@coredin/shared";

export class FeedService {
  constructor(private readonly http: HttpService) {}

  async get(id: number): Promise<PostDetailDTO> {
    return this.http.get("posts/" + id);
  }

  async getFeed(): Promise<PostDTO[]> {
    return this.http.get("posts");
  }

  async getUserFeed(user: string): Promise<PostDTO[]> {
    return this.http.get("posts/user/" + user);
  }

  async publish(createDTO: CreatePostDTO): Promise<void> {
    return this.http.post("posts", createDTO);
  }

  async likePost(postId: number, liked: boolean): Promise<void> {
    return this.http.post("posts/" + postId + `/like`, { liked });
  }

  async deletePost(postId: number): Promise<void> {
    return this.http.delete("posts/" + postId);
  }
}
