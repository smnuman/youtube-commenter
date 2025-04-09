import 'dart:convert';

class Comment {
  final String videoId;
  final String comment_id;
  final String author;
  final String authorChannelId;
  final String text;
  final int likeCount;
  final DateTime publishedAt;
  final List<Reply> replies;
  final bool repliedTo;
  final Map<String, dynamic> metadata;

  Comment({
    required this.videoId,
    required this.comment_id,
    required this.author,
    required this.authorChannelId,
    required this.text,
    required this.likeCount,
    required this.publishedAt,
    required this.replies,
    required this.repliedTo,
    required this.metadata,
  });

  factory Comment.fromJson(Map<String, dynamic> json) {
    return Comment(
      videoId: json['video_id'],
      comment_id: json['comment_id'],
      author: json['author'],
      authorChannelId: json['author_channel_id'],
      text: json['text'],
      likeCount: json['like_count'],
      publishedAt: DateTime.parse(json['published_at']),
      replies: (json['replies'] as List)
          .map((reply) => Reply.fromJson(reply))
          .toList(),
      repliedTo: json['replied_to'] ?? false,
      metadata: json['metadata'] ?? {},
    );
  }
}

class Reply {
  final String reply_id;
  final String parent_id;
  final String author;
  final String authorChannelId;
  final String text;
  final int likeCount;
  final DateTime publishedAt;
  final bool ai_generated;
  final String? ai_model;
  final Map<String, dynamic> metadata;

  Reply({
    required this.reply_id,
    required this.parent_id,
    required this.author,
    required this.authorChannelId,
    required this.text,
    required this.likeCount,
    required this.publishedAt,
    required this.ai_generated,
    this.ai_model,
    required this.metadata,
  });

  factory Reply.fromJson(Map<String, dynamic> json) {
    return Reply(
      reply_id: json['reply_id'],
      parent_id: json['parent_id'],
      author: json['author'],
      authorChannelId: json['author_channel_id'],
      text: json['text'],
      likeCount: json['like_count'],
      publishedAt: DateTime.parse(json['published_at']),
      ai_generated: json['ai_generated'] ?? false,
      ai_model: json['ai_model'],
      metadata: json['metadata'] ?? {},
    );
  }
}
