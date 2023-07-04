package rocks.kavin.reqwest4j;

import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.util.Map;
import java.util.concurrent.CompletableFuture;

public class ReqwestUtils {

    static {
        String arch = switch (System.getProperty("os.arch")) {
            case "aarch64" -> "aarch64";
            case "amd64" -> "x86_64";
            default -> throw new RuntimeException("Unsupported architecture");
        };

        File nativeFile;

        try {
            nativeFile = File.createTempFile("libreqwest", ".so");
            nativeFile.deleteOnExit();
        } catch (IOException e) {
            throw new RuntimeException(e);
        }

        final var cl = ReqwestUtils.class.getClassLoader();

        try (var stream = cl.getResourceAsStream("META-INF/natives/linux/" + arch + "/libreqwest.so")) {
            stream.transferTo(new FileOutputStream(nativeFile));
        } catch (IOException e) {
            throw new RuntimeException(e);
        }

        System.load(nativeFile.getAbsolutePath());
    }

    public static native void init(String proxy);

    public static native CompletableFuture<Response> fetch(String url, String method, byte[] body,
                                                          Map<String, String> headers);

}
