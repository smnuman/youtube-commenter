import 'dart:convert';

class User {
  final String id;
  final String name;
  final String? email;
  final String? profilePictureUrl;
  final DateTime createdAt;
  final DateTime updatedAt;
  final Map<String, dynamic> preferences;

  User({
    required this.id,
    required this.name,
    this.email,
    this.profilePictureUrl,
    required this.createdAt,
    required this.updatedAt,
    required this.preferences,
  });

  factory User.fromJson(String jsonString) {
    final data = json.decode(jsonString);
    return User.fromMap(data);
  }

  factory User.fromMap(Map<String, dynamic> json) {
    return User(
      id: json['id'],
      name: json['name'],
      email: json['email'],
      profilePictureUrl: json['profile_picture_url'],
      createdAt: DateTime.parse(json['created_at']),
      updatedAt: DateTime.parse(json['updated_at']),
      preferences: json['preferences'] ?? {},
    );
  }

  String toJson() {
    return json.encode({
      'id': id,
      'name': name,
      'email': email,
      'profile_picture_url': profilePictureUrl,
      'created_at': createdAt.toIso8601String(),
      'updated_at': updatedAt.toIso8601String(),
      'preferences': preferences,
    });
  }
}
