import 'dart:convert';
import 'package:flutter_dotenv/flutter_dotenv.dart';
import 'package:http/http.dart' as http;

import '../models/comment.dart';

class ApiService {
  final String baseUrl = dotenv.env['API_URL'] ?? 'http://localhost:3000';

  Future<List<Comment>> getComments(String videoId) async {
    final response = await http.get(
      Uri.parse('$baseUrl/api/comments/$videoId'),
      headers: {'Content-Type': 'application/json'},
    );

    if (response.statusCode == 200) {
      final List<dynamic> jsonData = json.decode(response.body);
      return jsonData.map((json) => Comment.fromJson(json)).toList();
    } else {
      throw Exception('Failed to load comments: ${response.statusCode}');
    }
  }

  Future<void> saveComments(String videoId, List<Comment> comments) async {
    final response = await http.post(
      Uri.parse('$baseUrl/api/comments/$videoId'),
      headers: {'Content-Type': 'application/json'},
      body: json.encode(comments),
    );

    if (response.statusCode != 201) {
      throw Exception('Failed to save comments: ${response.statusCode}');
    }
  }
}
