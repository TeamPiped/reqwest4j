package rocks.kavin.reqwest4j;

import java.io.File;
import java.io.IOException;
import java.util.Map;

public class ReqwestUtils {

    static {
        String arch;

        switch (System.getProperty("os.arch")) {
            case "aarch64":
                arch = "aarch64";
                break;
            case "amd64":
                arch = "x86_64";
                break;
            default:
                throw new RuntimeException("Unsupported architecture");
        }

        String fileName =
                System.getProperty("java.io.tmpdir") +
                        File.separatorChar +
                        "libreqwest_" + System.currentTimeMillis() + ".so";

        final var cl = ReqwestUtils.class.getClassLoader();

        try (var stream = cl.getResourceAsStream("META-INF/natives/linux/" + arch + "/libreqwest.so")) {
            stream.transferTo(new java.io.FileOutputStream(fileName));
        } catch (IOException e) {
            throw new RuntimeException(e);
        }

        System.load(fileName);
    }

    public static native Response fetch(String url, String method, byte[] body,
                                        Map<String, String> headers);

}
