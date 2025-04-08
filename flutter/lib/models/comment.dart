class Comment {
  final String videoId;
  final String commentId;
  final String author;
  final String text;
  final int likeCount;
  final DateTime publishedAt;
  final List<Reply> replies;

  Comment({
    required this.videoId,
    required this.commentId,
    required this.author,
    required this.text,
    required this.likeCount,
    required this.publishedAt,
    required this.replies,
  });

  factory Comment.fromJson(Map<String, dynamic> json) {
    return Comment(
      videoId: json['video_id'],
      commentId: json['comment_id'],
      author: json['author'],
      text: json['text'],
      likeCount: json['like_count'],
      publishedAt: DateTime.parse(json['published_at']),
      replies: (json['replies'] as List)
          .map((reply) => Reply.fromJson(reply))
          .toList(),
    );
  }
}

class Reply {
  final String replyId;
  final String author;
  final String text;
  final int likeCount;
  final DateTime publishedAt;

  Reply({
    required this.replyId,
    required this.author,
    required this.text,
    required this.likeCount,
    required this.publishedAt,
  });

  factory Reply.fromJson(Map<String, dynamic> json) {
    return Reply(
      replyId: json['reply_id'],
      author: json['author'],
      text: json['text'],
      likeCount: json['like_count'],
      publishedAt: DateTime.parse(json['published_at']),
    );
  }
}
