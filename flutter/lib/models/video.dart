class Video {
  final String id;
  final String title;
  final String description;
  final DateTime publishedAt;
  final String thumbnailUrl;

  Video({
    required this.id,
    required this.title,
    required this.description,
    required this.publishedAt,
    required this.thumbnailUrl,
  });

  factory Video.fromJson(Map<String, dynamic> json) {
    return Video(
      id: json['id'],
      title: json['title'],
      description: json['description'],
      publishedAt: DateTime.parse(json['published_at']),
      thumbnailUrl: json['thumbnail_url'],
    );
  }
}
