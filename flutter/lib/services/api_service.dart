import 'dart:convert';
import 'package:flutter_dotenv/flutter_dotenv.dart';
import 'package:http/http.dart' as http;

import '../models/comment.dart';
import '../models/video.dart';

class ApiService {
  final String baseUrl = dotenv.env['API_URL'] ?? 'http://localhost:3000';

  // Comments API
  Future<List<Comment>> getComments(String videoId, String sessionId) async {
    final response = await http.get(
      Uri.parse('$baseUrl/api/comments/$videoId'),
      headers: {
        'Content-Type': 'application/json',
        'x-session-id': sessionId,
      },
    );

    if (response.statusCode == 200) {
      final List<dynamic> jsonData = json.decode(response.body);
      return jsonData.map((json) => Comment.fromJson(json)).toList();
    } else {
      throw Exception('Failed to load comments: ${response.statusCode}');
    }
  }

  // Videos API
  Future<List<Video>> getVideos(String sessionId) async {
    final response = await http.get(
      Uri.parse('$baseUrl/api/videos'),
      headers: {
        'Content-Type': 'application/json',
        'x-session-id': sessionId,
      },
    );

    if (response.statusCode == 200) {
      final List<dynamic> jsonData = json.decode(response.body);
      return jsonData.map((json) => Video.fromJson(json)).toList();
    } else {
      throw Exception('Failed to load videos: ${response.statusCode}');
    }
  }

  // Reply API
  Future<GenerateReplyResponse> generateReply(String commentId, String tone, String sessionId) async {
    final response = await http.post(
      Uri.parse('$baseUrl/api/reply/generate'),
      headers: {
        'Content-Type': 'application/json',
        'x-session-id': sessionId,
      },
      body: json.encode({
        'comment_id': commentId,
        'tone': tone,
      }),
    );

    if (response.statusCode == 200) {
      final jsonData = json.decode(response.body);
      return GenerateReplyResponse.fromJson(jsonData);
    } else {
      throw Exception('Failed to generate reply: ${response.statusCode}');
    }
  }

  Future<void> postReply(String commentId, String replyText, bool aiGenerated, String? aiModel, String sessionId) async {
    final response = await http.post(
      Uri.parse('$baseUrl/api/reply/post'),
      headers: {
        'Content-Type': 'application/json',
        'x-session-id': sessionId,
      },
      body: json.encode({
        'comment_id': commentId,
        'reply_text': replyText,
        'ai_generated': aiGenerated,
        if (aiModel != null) 'ai_model': aiModel,
      }),
    );

    if (response.statusCode != 200) {
      throw Exception('Failed to post reply: ${response.statusCode}');
    }
  }

  // History API
  Future<List<InteractionRecord>> getHistory(String sessionId, {int limit = 100}) async {
    final response = await http.get(
      Uri.parse('$baseUrl/api/history?limit=$limit'),
      headers: {
        'Content-Type': 'application/json',
        'x-session-id': sessionId,
      },
    );

    if (response.statusCode == 200) {
      final List<dynamic> jsonData = json.decode(response.body);
      return jsonData.map((json) => InteractionRecord.fromJson(json)).toList();
    } else {
      throw Exception('Failed to load history: ${response.statusCode}');
    }
  }
}

class GenerateReplyResponse {
  final String replyText;
  final String model;

  GenerateReplyResponse({
    required this.replyText,
    required this.model,
  });

  factory GenerateReplyResponse.fromJson(Map<String, dynamic> json) {
    return GenerateReplyResponse(
      replyText: json['reply_text'],
      model: json['model'],
    );
  }
}

class InteractionRecord {
  final String id;
  final String userId;
  final String videoId;
  final String commentId;
  final String? replyId;
  final String interactionType;
  final DateTime timestamp;
  final Map<String, dynamic> data;

  InteractionRecord({
    required this.id,
    required this.userId,
    required this.videoId,
    required this.commentId,
    this.replyId,
    required this.interactionType,
    required this.timestamp,
    required this.data,
  });

  factory InteractionRecord.fromJson(Map<String, dynamic> json) {
    return InteractionRecord(
      id: json['id'],
      userId: json['user_id'],
      videoId: json['video_id'],
      commentId: json['comment_id'],
      replyId: json['reply_id'],
      interactionType: json['interaction_type'],
      timestamp: DateTime.parse(json['timestamp']),
      data: json['data'] ?? {},
    );
  }
}
