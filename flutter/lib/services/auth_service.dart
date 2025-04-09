import 'dart:convert';
import 'package:flutter_dotenv/flutter_dotenv.dart';
import 'package:http/http.dart' as http;
import 'package:url_launcher/url_launcher.dart';

class AuthService {
  final String baseUrl = dotenv.env['API_URL'] ?? 'http://localhost:3000';

  Future<String> getAuthUrl() async {
    final response = await http.get(
      Uri.parse('$baseUrl/api/auth/url'),
      headers: {'Content-Type': 'application/json'},
    );

    if (response.statusCode == 200) {
      final jsonData = json.decode(response.body);
      return jsonData['url'];
    } else {
      throw Exception('Failed to get auth URL: ${response.statusCode}');
    }
  }

  Future<void> launchAuthFlow() async {
    final url = await getAuthUrl();
    final uri = Uri.parse(url);
    if (await canLaunchUrl(uri)) {
      await launchUrl(uri, mode: LaunchMode.externalApplication);
    } else {
      throw Exception('Could not launch $url');
    }
  }
}
