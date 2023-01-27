package rocks.kavin.reqwest4j;

import java.util.Map;

public record Response(int status, Map<String, String> headers, byte[] body, String finalUrl) {

}
